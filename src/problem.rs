
#[derive(Debug, Clone)]
pub struct TSP {
    pub name: String           , 
    pub ps  : Box<[(f64, f64)]>, 
    pub cost: Box<[u32]>       , // linearized square matrix
    pub len : usize            ,
}

pub type Point = u32;
pub type Tour  = Box<[Point]>;

impl TSP {
    // AI Generated
    pub fn parser(input: &str) -> Option<Self> {
        let mut lines = input.lines().map(str::trim);

        let mut name = None;
        let mut dimension = None;
        let mut edge_type = None;
        let mut found_section = false;

        // Header
        for line in lines.by_ref() {
            if line == "NODE_COORD_SECTION" {
                found_section = true;
                break;
            }

            if let Some((key, value)) = line.split_once(':') {
                let value = value.trim();

                match key.trim() {
                    "NAME" => name = Some(value.to_owned()),
                    "DIMENSION" => {
                        dimension = value.parse::<usize>().ok();
                    }
                    "TYPE" if value != "TSP" => return None,
                    "EDGE_WEIGHT_TYPE" => edge_type = Some(value),
                    _ => {}
                }
            }
        }

        if !found_section || edge_type != Some("EUC_2D") {
            return None;
        }

        let dimension = dimension?;
        let mut ps = vec![(0.0, 0.0); dimension];
        let mut seen = vec![false; dimension];

        // Coordinates
        for line in lines {
            if line == "EOF" {
                break;
            }

            if line.is_empty() {
                continue;
            }

            let mut parts = line.split_whitespace();

            let id: usize = parts.next()?.parse().ok()?;
            let x: f64 = parts.next()?.parse().ok()?;
            let y: f64 = parts.next()?.parse().ok()?;

            if id == 0 || id > dimension || seen[id - 1] {
                return None;
            }

            ps[id - 1] = (x, y);
            seen[id - 1] = true;
        }

        if seen.iter().any(|&v| !v) {
            return None;
        }

        // Symmetric distance matrix
        let size = dimension.checked_mul(dimension)?;
        let mut costs = vec![0u32; size];

        for i in 0..dimension {
            for j in (i + 1)..dimension {
                let d = euclidean_diff(&ps, i, j);

                costs[i * dimension + j] = d;
                costs[j * dimension + i] = d;
            }
        }

        Some(Self {
            name: name?,
            ps: ps.into_boxed_slice(),
            cost: costs.into_boxed_slice(),
            len: dimension,
        })
    }

    #[inline]
    pub fn distance(&self, i: usize, j: usize) -> u32 {
        self.cost[i * self.len + j]
    }
}

// AI Generated
#[inline]
pub fn euclidean_diff(
    points: &[(f64, f64)],
    i: usize,
    j: usize,
) -> u32 {
    let (x1, y1) = points[i];
    let (x2, y2) = points[j];

    let dx = x1 - x2;
    let dy = y1 - y2;

    ((dx * dx + dy * dy).sqrt() + 0.5) as u32
}
