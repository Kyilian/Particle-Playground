use glam::Vec2;

use crate::particle::Particle;

#[derive(Clone, Copy)]
pub struct Quadrant {
    pub center: Vec2,
    pub size: f32,
}

impl Quadrant {
    //finds the smallest quadrat for the particles
    pub fn new(particle: &[Particle]) -> Self {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for particles in particle {
            min_x = min_x.min(particles.pos.x);
            min_y = min_y.min(particles.pos.y);
            max_x = max_x.max(particles.pos.x);
            max_y = max_y.max(particles.pos.y);
        }

        let center = Vec2::new(min_x + max_x, min_y + max_y) * 0.5;
        let size = (max_x - min_x).max(max_y - min_y);

        Self { center, size }
    }

    //find the quadrant from 0 to 3
    pub fn find_quadrant(&self, pos: Vec2) -> usize {
        ((pos.y > self.center.y) as usize) << 1 | (pos.x > self.center.x) as usize
    }
    //split the quadrant and move the center into the new qadrant
    pub fn into_quadrant(mut self, quadrant: usize) -> Self {
        self.size *= 0.5;
        self.center.x += ((quadrant & 1) as f32 - 0.5) * self.size;
        self.center.y += ((quadrant >> 1) as f32 - 0.5) * self.size;
        self
    }

    //Split the quadrant into 4 parts
    pub fn subdivide(&self) -> [Quadrant; 4] {
        [0, 1, 2, 3].map(|i| self.into_quadrant(i))
    }
}
#[derive(Clone, Copy)]
pub struct Node {
    pub children: usize,
    pub next: usize,
    pub pos: Vec2,
    pub mass: f32,
    pub quadrant: Quadrant,
}

impl Node {
    pub fn new(next: usize, quadrant: Quadrant) -> Self {
        Self {
            children: 0,
            next,
            pos: Vec2::ZERO,
            mass: 0.0,
            quadrant,
        }
    }
    pub fn is_leaf(&self) -> bool {
        self.children == 0
    }

    pub fn is_branch(&self) -> bool {
        self.children != 0
    }

    pub fn is_empty(&self) -> bool {
        self.mass == 0.0
    }
}

#[derive(Clone)]
pub struct Quadtree {
    pub t_sq: f32,
    pub e_sq: f32,
    pub nodes: Vec<Node>,
    pub parents: Vec<usize>,
}

impl Quadtree {
    pub const ROOT: usize = 0;

    //not normal heap quadtree, use Vec to store
    pub fn new(theta: f32, epsion: f32) -> Self {
        Self {
            t_sq: theta * theta,
            e_sq: epsion * epsion,
            nodes: Vec::new(),
            parents: Vec::new(),
        }
    }

    pub fn clear(&mut self, quad: Quadrant) {
        self.nodes.clear();
        self.parents.clear();
        self.nodes.push(Node::new(0, quad));
    }

    pub fn subdivide(&mut self, node: usize) -> usize {
        self.parents.push(node);
        let children = self.nodes.len();

        let nexts = [
            //root is always at index 0
            children + 1,
            children + 2,
            children + 3,
            self.nodes[node].next,
        ];

        let quads = self.nodes[node].quadrant.subdivide();
        for i in 0..4 {
            self.nodes.push(Node::new(nexts[i], quads[i]));
        }

        return children;
    }

    pub fn insert(mut self, pos: Vec2, mass: f32) {
        let mut node = Self::ROOT;

        while self.nodes[node].is_branch() {
            let quadrant = self.nodes[node].quadrant.find_quadrant(pos);
            node = self.nodes[node].children + quadrant;
        }

        if self.nodes[node].is_empty() {
            self.nodes[node].pos = pos;
            self.nodes[node].mass = mass;
            return;
        }

        let (p, m) = (self.nodes[node].pos, self.nodes[node].mass);

        //combining the masses if exact same position
        if pos == p {
            self.nodes[node].mass += mass;
            return;
        }

        //if not empty create new quadrants until we found an empty one
        loop {
            let children = self.subdivide(node);

            let quadrant1 = self.nodes[node].quadrant.find_quadrant(p);
            let quadrant2 = self.nodes[node].quadrant.find_quadrant(pos);

            if quadrant1 == quadrant2 {
                node = children + quadrant1;
            } else {
                //if different quadrant, add particles into quadrant
                let n1 = children + quadrant1;
                let n2 = children + quadrant2;

                self.nodes[n1].pos = p;
                self.nodes[n1].mass = m;
                self.nodes[n2].pos = pos;
                self.nodes[n2].mass = mass;
                return;
            }
        }
    }

    //to calculate the masses of every quadrant, we need to traverse the vec in reverse order
    //calculating the center of mass for every quadrant, so we can see a quadrant as one big particle
    pub fn propagate(&mut self) {
        for &node in self.parents.iter().rev() {
            //start with the smallest quadrant to calculate the mass for the upper quadrant
            let i = self.nodes[node].children;

            self.nodes[node].pos = self.nodes[i].pos * self.nodes[i].mass
                + self.nodes[i + 1].pos * self.nodes[i + 1].mass
                + self.nodes[i + 2].pos * self.nodes[i + 2].mass
                + self.nodes[i + 3].pos * self.nodes[i + 3].mass;

            self.nodes[node].mass = self.nodes[i].mass
                + self.nodes[i + 1].mass
                + self.nodes[i + 2].mass
                + self.nodes[i + 3].mass;

            let mass = self.nodes[node].mass;
            self.nodes[node].pos /= mass;
        }
    }

    pub fn acc(&self, pos: Vec2) -> Vec2 {
        let mut acc = Vec2::ZERO;

        let mut node = Self::ROOT;

        loop {
            let n = &self.nodes[node];
            let d = n.pos - pos;
            let d_sq = (d.x * d.x) + (d.y * d.y);

            if n.is_leaf() || n.quadrant.size * n.quadrant.size < d_sq * self.t_sq {
                let denom = (d_sq + self.e_sq) * d_sq.sqrt();
                acc += d * (n.mass / denom).min(f32::MAX);

                if n.next == 0 {
                    break;
                }
                node = n.next;
            } else {
                node = n.children;
            }
        }
        acc
    }
}
