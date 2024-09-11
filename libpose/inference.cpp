
#include <cstdio>
#include <jetson-utils/commandLine.h>
#include <pose.hpp>

int main(int argc, char** argv) {

    commandLine cmdLine{argc, argv};
    initialize();
    
    inference_start("/dev/video0");
    for(;;) {
        pose_data data = inference_step();
        for (int i = 0; i < data.detected_kps; i++) {
            keypoint kp = data.kps[i];
            printf("%2d | x: %3f, y: %3f\n", i, kp.x, kp.y);
        }
    }
    inference_end();

    shutdown();
    return 0;
}