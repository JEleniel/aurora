use super::*;

#[test]
fn compress_polyline_removes_duplicates_and_collinear_points() {
	let points = vec![
		geom::PointF { x: 0.0, y: 0.0 },
		geom::PointF { x: 0.0, y: 0.0 },
		geom::PointF { x: 10.0, y: 0.0 },
		geom::PointF { x: 20.0, y: 0.0 },
		geom::PointF { x: 20.0, y: 10.0 },
	];
	let out = compress_polyline(points);
	assert_eq!(out.len(), 3);
	assert!((out[0].x - 0.0).abs() < 0.01);
	assert!((out[0].y - 0.0).abs() < 0.01);
	assert!((out[1].x - 20.0).abs() < 0.01);
	assert!((out[1].y - 0.0).abs() < 0.01);
	assert!((out[2].x - 20.0).abs() < 0.01);
	assert!((out[2].y - 10.0).abs() < 0.01);
}

#[test]
fn arrowhead_uses_last_point_as_tip() {
	let points = vec![
		geom::PointF { x: 0.0, y: 0.0 },
		geom::PointF { x: 10.0, y: 0.0 },
	];
	let arrow = arrowhead(points.as_slice());
	assert!((arrow[0].x - 10.0).abs() < 0.01);
	assert!((arrow[0].y - 0.0).abs() < 0.01);
}

#[test]
fn finalize_route_sets_bounds_and_arrow() {
	let mut route = Route {
		edge_id: "e".to_string(),
		source_id: "a".to_string(),
		target_id: "b".to_string(),
		points: vec![
			geom::PointF { x: 0.0, y: 0.0 },
			geom::PointF { x: 100.0, y: 0.0 },
		],
		junction_ids: Vec::new(),
		track_id: "t".to_string(),
		arrow: [geom::PointF { x: 0.0, y: 0.0 }; 3],
		bounds: geom::Bounds::empty(),
	};
	finalize_route(&mut route);
	assert!(route.bounds.min_x <= route.arrow[0].x);
	assert!(route.bounds.max_x >= route.arrow[0].x);
}

#[test]
fn jump_path_uses_doubled_radius() {
	let path = jump_path(geom::PointF { x: 50.0, y: 60.0 });
	assert_eq!(path, "M 34.00 60.00 Q 50.00 44.00 66.00 60.00");
}
