use prepose::PoseEstimator;

fn main() {
    let mut pose = PoseEstimator::new(
        "../network/pose_resnet18_body.onnx", 
        "../network/human_pose.json", 
        "../network/colors.txt"
    );
}
