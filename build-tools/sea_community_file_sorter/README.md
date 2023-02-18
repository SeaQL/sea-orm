## SeaOrm community file sorter
This is a simple script to sort the community file on the SeaORM repository.

### The Minimum supported Rust version (MSRV)
Is 1.56.1

### Usage
You can sort the community file by running the following command:
```bash
cargo run --release -- <path of community file>
```
Or if you want to check if the file is sorted, you can run the following command:
```bash
cargo run --release -- <path of community file> --check
```

### Note
This script work with specific format of the community file. You can see it in the main file of this project here [main.rs](./src/main.rs).

### Example
<img height="350" width="450" src="https://i.suar.me/aLVj9/l">

### License
This project is licensed under the MIT license.