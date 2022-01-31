# mlsmpm-particles-rs

This repository simulates and renders fluid particles in two dimensions, in Rust.  
My matching implementation in Go is [mlsmpm-particles-go](https://github.com/robkau/mlsmpm-particles-go).

The simulation uses MLS-MPM algorithm (Moving Least Squares Material Point Method).   
Both repositories were implemented by following [nialltl's article on MLS-MPM](https://nialltl.neocities.org/articles/mpm_guide.html) and [matching example code](https://github.com/nialltl/incremental_mpm).

Library [bevy](https://github.com/bevyengine/bevy) is used to render the output to a window and provide an Entity Component System with parallel processing.  
Liby99's [mpm-rs](https://github.com/Liby99/mpm-rs) project inspired me to model MPM simulation inside an ECS.  
Unlike mlsmpm-particles-go, this implementation in rust is parallelized and can fully utilize all CPU cores when enough particles are spawned.    
The Bevy ECS is really ergonomic, all it took to implement parallel processing was calling the par_for_each methods and the rest happened automatically.   
