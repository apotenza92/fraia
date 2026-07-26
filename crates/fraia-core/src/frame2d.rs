use crate::types::{
    Combo2D, ElementResult2D, FrameModel2D, NodeResult2D, SolveMetrics2D, SolveResult2D,
};
use crate::utils::max_abs;
use anyhow::{Result, bail};
use std::collections::HashMap;

pub fn solve_frame_2d(model: &FrameModel2D, combo: &Combo2D) -> Result<SolveResult2D> {
    let node_index: HashMap<&str, usize> = model
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();
    let dof_count = model.nodes.len() * 3;
    let mut k_global = zero_matrix(dof_count, dof_count);
    let mut element_data: Vec<ResolvedElement> = vec![];

    for element in &model.elements {
        let ni = &model.nodes[*node_index.get(element.i.as_str()).unwrap()];
        let nj = &model.nodes[*node_index.get(element.j.as_str()).unwrap()];
        let l = ((nj.x - ni.x).powi(2) + (nj.y - ni.y).powi(2)).sqrt();
        let c = (nj.x - ni.x) / l;
        let s = (nj.y - ni.y) / l;
        let k_local = local_frame_stiffness(
            element.material.e,
            element.section.area,
            element.section.i,
            l,
        );
        let t = transformation(c, s);
        let k_elem = multiply(&multiply(&transpose(&t), &k_local), &t);
        let dofs = element_dofs(
            *node_index.get(element.i.as_str()).unwrap(),
            *node_index.get(element.j.as_str()).unwrap(),
        );
        assemble(&mut k_global, &k_elem, &dofs);
        element_data.push(ResolvedElement {
            element,
            l,
            k_local,
            t,
            dofs,
        });
    }

    let mut load_vectors: HashMap<&str, Vec<f64>> = HashMap::new();
    for lc in &model.load_cases {
        let mut f = vec![0.0; dof_count];
        for load in &lc.nodal_loads {
            let idx = *node_index.get(load.node.as_str()).unwrap();
            f[idx * 3] += load.fx;
            f[idx * 3 + 1] += load.fy;
            f[idx * 3 + 2] += load.mz;
        }
        load_vectors.insert(lc.id.as_str(), f);
    }

    let mut f_combo = vec![0.0; dof_count];
    for (case_id, factor) in &combo.factors {
        if let Some(f) = load_vectors.get(case_id.as_str()) {
            for i in 0..dof_count {
                f_combo[i] += factor * f[i];
            }
        }
    }

    let mut restraints = vec![false; dof_count];
    for support in &model.supports {
        let index = *node_index.get(support.node.as_str()).unwrap();
        if support.ux {
            restraints[index * 3] = true;
        }
        if support.uy {
            restraints[index * 3 + 1] = true;
        }
        if support.rz {
            restraints[index * 3 + 2] = true;
        }
    }

    let mut free = vec![];
    for (i, restrained) in restraints.iter().enumerate() {
        if !restrained {
            free.push(i);
        }
    }
    if free.is_empty() {
        bail!("No free DOFs in model");
    }

    let k_ff = submatrix(&k_global, &free, &free);
    let f_f: Vec<f64> = free.iter().map(|&i| f_combo[i]).collect();
    let u_f = solve_linear_system(k_ff, f_f)?;
    let mut u = vec![0.0; dof_count];
    for (i, dof) in free.iter().enumerate() {
        u[*dof] = u_f[i];
    }

    let reactions: Vec<f64> = mat_vec(&k_global, &u)
        .iter()
        .enumerate()
        .map(|(i, v)| v - f_combo[i])
        .collect();

    let element_results: Vec<ElementResult2D> = element_data
        .iter()
        .map(|resolved| {
            let u_global: Vec<f64> = resolved.dofs.iter().map(|&d| u[d]).collect();
            let u_local = mat_vec(&resolved.t, &u_global);
            let f_local = mat_vec(&resolved.k_local, &u_local);
            let axial = f_local[0].abs().max(f_local[3].abs());
            let moment = f_local[2].abs().max(f_local[5].abs());
            let c_depth = resolved.element.section.depth / 2.0;
            let stress = axial / resolved.element.section.area
                + (moment * c_depth) / resolved.element.section.i;
            ElementResult2D {
                id: resolved.element.id.clone(),
                role: resolved.element.role.clone(),
                length_m: resolved.l,
                local_end_forces: f_local.clone(),
                axial_n: axial,
                moment_nm: moment,
                utilization: stress / resolved.element.material.fy,
                stress_pa: stress,
            }
        })
        .collect();

    let node_results: Vec<NodeResult2D> = model
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| NodeResult2D {
            id: node.id.clone(),
            x: node.x,
            y: node.y,
            ux_m: u[i * 3],
            uy_m: u[i * 3 + 1],
            rz_rad: u[i * 3 + 2],
            rxn_fx_n: reactions[i * 3],
            rxn_fy_n: reactions[i * 3 + 1],
            rxn_mz_nm: reactions[i * 3 + 2],
        })
        .collect();

    Ok(SolveResult2D {
        combo: combo.clone(),
        node_results: node_results.clone(),
        element_results: element_results.clone(),
        metrics: SolveMetrics2D {
            max_utilization: max_abs(
                &element_results
                    .iter()
                    .map(|e| e.utilization)
                    .collect::<Vec<_>>(),
            ),
            max_ux_m: max_abs(&node_results.iter().map(|n| n.ux_m).collect::<Vec<_>>()),
            max_uy_m: max_abs(&node_results.iter().map(|n| n.uy_m).collect::<Vec<_>>()),
            max_reaction_n: max_abs(&reactions),
        },
    })
}

struct ResolvedElement<'a> {
    element: &'a crate::types::FrameElement2D,
    l: f64,
    k_local: Vec<Vec<f64>>,
    t: Vec<Vec<f64>>,
    dofs: Vec<usize>,
}

fn local_frame_stiffness(e: f64, a: f64, i: f64, l: f64) -> Vec<Vec<f64>> {
    let aa = (e * a) / l;
    let b = (12.0 * e * i) / l.powi(3);
    let c = (6.0 * e * i) / l.powi(2);
    let d = (4.0 * e * i) / l;
    let ee = (2.0 * e * i) / l;
    vec![
        vec![aa, 0.0, 0.0, -aa, 0.0, 0.0],
        vec![0.0, b, c, 0.0, -b, c],
        vec![0.0, c, d, 0.0, -c, ee],
        vec![-aa, 0.0, 0.0, aa, 0.0, 0.0],
        vec![0.0, -b, -c, 0.0, b, -c],
        vec![0.0, c, ee, 0.0, -c, d],
    ]
}

fn transformation(c: f64, s: f64) -> Vec<Vec<f64>> {
    vec![
        vec![c, s, 0.0, 0.0, 0.0, 0.0],
        vec![-s, c, 0.0, 0.0, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        vec![0.0, 0.0, 0.0, c, s, 0.0],
        vec![0.0, 0.0, 0.0, -s, c, 0.0],
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
    ]
}

fn element_dofs(i: usize, j: usize) -> Vec<usize> {
    vec![i * 3, i * 3 + 1, i * 3 + 2, j * 3, j * 3 + 1, j * 3 + 2]
}

fn zero_matrix(rows: usize, cols: usize) -> Vec<Vec<f64>> {
    vec![vec![0.0; cols]; rows]
}

fn assemble(k: &mut [Vec<f64>], e: &[Vec<f64>], dofs: &[usize]) {
    for (i, &ri) in dofs.iter().enumerate() {
        for (j, &cj) in dofs.iter().enumerate() {
            k[ri][cj] += e[i][j];
        }
    }
}

fn transpose(a: &[Vec<f64>]) -> Vec<Vec<f64>> {
    (0..a[0].len())
        .map(|i| a.iter().map(|row| row[i]).collect())
        .collect()
}

fn multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let rows = a.len();
    let cols = b[0].len();
    let inner = b.len();
    let mut out = zero_matrix(rows, cols);
    for i in 0..rows {
        for j in 0..cols {
            let mut total = 0.0;
            for k in 0..inner {
                total += a[i][k] * b[k][j];
            }
            out[i][j] = total;
        }
    }
    out
}

fn mat_vec(a: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
    a.iter()
        .map(|row| row.iter().enumerate().map(|(i, value)| value * x[i]).sum())
        .collect()
}

fn submatrix(a: &[Vec<f64>], rows: &[usize], cols: &[usize]) -> Vec<Vec<f64>> {
    rows.iter()
        .map(|&r| cols.iter().map(|&c| a[r][c]).collect())
        .collect()
}

fn solve_linear_system(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Result<Vec<f64>> {
    let n = a.len();
    for col in 0..n {
        let mut pivot = col;
        for row in (col + 1)..n {
            if a[row][col].abs() > a[pivot][col].abs() {
                pivot = row;
            }
        }
        if a[pivot][col].abs() < 1e-12 {
            bail!("Singular stiffness matrix. Structure may be unstable.");
        }
        if pivot != col {
            a.swap(pivot, col);
            b.swap(pivot, col);
        }
        let pivot_val = a[col][col];
        for j in col..n {
            a[col][j] /= pivot_val;
        }
        b[col] /= pivot_val;
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = a[row][col];
            for j in col..n {
                a[row][j] -= factor * a[col][j];
            }
            b[row] -= factor * b[col];
        }
    }
    Ok(b)
}
