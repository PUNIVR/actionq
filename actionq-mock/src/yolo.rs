use ndarray::{s, Array, Axis, IxDyn};
use ort::{
    execution_providers::{
        CUDAExecutionProvider, 
        TensorRTExecutionProvider
    }, 
    session::{
        builder::GraphOptimizationLevel, 
        Session
    }
};

#[derive(Debug, Clone)]
pub struct Point2 {
	pub x: f32,
	pub y: f32,
	pub c: f32,
}

#[derive(Debug, Clone)]
pub struct BBox {
	xmin: f32,
    ymin: f32,
    w: f32,
    h: f32,
    id: usize,
    confidence: f32,
}

impl BBox {
	
	pub fn area(&self) -> f32 {
		self.w * self.h
	}
	
	pub fn intersection_area(&self, other: &BBox) -> f32 {
        let r = (self.xmin + self.w).min(other.xmin + other.w);
        let b = (self.ymin + self.h).min(other.ymin + other.h);
		let l = self.xmin.max(other.xmin);
        let t = self.ymin.max(other.ymin);
		(r - l + 1.).max(0.) * (b - t + 1.).max(0.)
    }

	pub fn union(&self, other: &BBox) -> f32 {
        self.area() + other.area() - self.intersection_area(other)
    }

	pub fn iou(&self, other: &BBox) -> f32 {
		self.intersection_area(other) / self.union(other)
	}
}

fn non_max_suppression(xs: &mut Vec<(BBox, Vec<Point2>)>, iou_threshold: f32) {
    xs.sort_by(|b1, b2| b2.0.confidence.partial_cmp(&b1.0.confidence).unwrap());

    let mut current_index = 0;
    for index in 0..xs.len() {
        let mut drop = false;
        for prev_index in 0..current_index {
            let iou = xs[prev_index].0.iou(&xs[index].0);
            if iou > iou_threshold {
                drop = true;
                break;
            }
        }
        if !drop {
            xs.swap(current_index, index);
            current_index += 1;
        }
    }

    xs.truncate(current_index);
}
	
pub fn parse_output(output: &mut Array<f32, IxDyn>) -> Vec<(BBox, Vec<Point2>)>{
	
	const BBOX_OFFSET: usize = 5; // cx-cy-w-h-conf
	const KPT_OFFSET: usize = 3;  // x-y-conf 
	const KPT_NUM: usize = 17;

	// Transpose output to be 8400x56 from initial 1x56x8400
	let mut output = output.index_axis(Axis(0), 0);
	output.swap_axes(0, 1);
	
	let mut data: Vec<(BBox, Vec<Point2>)> = Vec::new();
	for i in 0..8400 /* For each box */ {
		let result = output.index_axis(Axis(0), i);

		// Get bounding box
		let bbox = result.slice(s![0..BBOX_OFFSET]);
		let bbox = {

			let cx = bbox[0];
			let cy = bbox[1];
			let bw = bbox[2];
			let bh = bbox[3];
			let bc = bbox[4];

			// Skip box if confidence is low
			if bc < 0.5 { continue; }

			let bx = cx - bw / 2.;
			let by = cy - bh / 2.;
			BBox {
				xmin: bx,
				ymin: by,
				w: bw,
				h: bh,
				id: i,
				confidence: bc,
			}
		};
		
		let kpts = result.slice(s![BBOX_OFFSET..]);
		let kpts = {
			(0..KPT_NUM).into_iter()
				.map(|i| {
					Point2 {
						x: kpts[i * KPT_OFFSET + 0],
						y: kpts[i * KPT_OFFSET + 1],
						c: kpts[i * KPT_OFFSET + 2]
					}
				}).collect()
		};

		non_max_suppression(&mut data, 0.45);
		data.push((bbox, kpts));
	}

	// Return skeletons in order of confidence
	data.sort_by(|a, b| {
		a.0.confidence.partial_cmp(&b.0.confidence).unwrap().reverse()
	});
	data
}

pub fn load_model(filepath: &str) -> ort::Result<Session> {
    Session::builder()?
		.with_optimization_level(GraphOptimizationLevel::Level1)?
		.with_execution_providers([
			TensorRTExecutionProvider::default()
				.with_device_id(0)
				.with_engine_cache(true)
				.with_cuda_graph(true)
                .build(),
            CUDAExecutionProvider::default()
				.with_device_id(0)
                .build()
		])?
        .commit_from_file(filepath)
}