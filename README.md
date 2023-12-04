# VRow

VRow is a software to communicate between Concept 2 Ergometers and other applications. Therefore it exposes data received from the ergometer to a UDP socket. Using this data you can build, for example, VR exergames or other applications to include the data from the ergometer.

## Build the project

Requirements:
- Rust and Cargo installed
- Linux or MacOS (Windows is not supported due to BLE limitations)

To create a development build, clone the repo and build the project using `cargo build`.

## How to cite VRow

Below are the BibTex entries to cite VRow.

```bibtex
@inproceedings{10.1145/3626705.3631785,
author = {Sch\"{o}n, Dominik and Kosch, Thomas and von Willich, Julius and M\"{u}hlh\"{a}user, Max and G\"{u}nther, Sebastian},
title = {VRow-VRow-VRow-Your-Boat: A Toolkit for Integrating Commodity Ergometers in Virtual Reality Experiences},
year = {2023},
isbn = {9798400709210},
publisher = {Association for Computing Machinery},
address = {New York, NY, USA},
url = {https://doi.org/10.1145/3626705.3631785},
doi = {10.1145/3626705.3631785},
abstract = {Exergames, video games designed to blend entertainment with physical activity, aim to improve users’ physical fitness by combining gaming with exercise. However, integrating exercise equipment, such as rowers, bikes, and ski ergometers into Virtual Reality (VR) environments remains challenging. In this poster, we introduce a toolkit that simplifies the integration of ergometers into Unity-based projects. Researchers can access detailed ergometer data for logging and use inside their projects, while our toolkit handles tedious tasks, like connection-handling or parsing. VRow offers valuable support for creating immersive and interactive fitness experiences.},
booktitle = {Proceedings of the 22nd International Conference on Mobile and Ubiquitous Multimedia},
pages = {483–485},
numpages = {3},
keywords = {Rower, Virtual Reality, Ergometer, Exergame},
location = {, Vienna, Austria, },
series = {MUM '23}
}
```
