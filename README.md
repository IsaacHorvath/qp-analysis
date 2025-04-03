# House Words
A web tool to generate graphs of word usage by Canadian MPs and Ontario MPPs. Check out the federal version hosted [here](https://housewords.chunkerbunker.cc/) and the Ontario version [here](https://queens-park.chunkerbunker.cc/).

## Installation
Unfortunately, there will be no way to install a *working* version of this tool yourself until I provide the data I have scraped, or the scrapers I used to generate the data. For now, here are the steps to get the backend and frontend up. Running the tool using the dev script will return dummy data by default.

These steps assume you have working and up to date rustup and cargo installations on some flavour of Linux.

### Step 1. Clone Repository
Clone this repository and enter it.

```sh
git clone https://github.com/IsaacHorvath/qp-analysis.git
cd qp-analysis
```

### Step 2. Install Trunk and Wasm Target
Install `trunk`. trunk allows us to pack the Yew frontend into a wasm binary, and provides a simple server to serve that binary when we're in a dev environment.

```sh
cargo install trunk
rustup rustup target add wasm32-unknown-unknown
```

### Step 3. Set Environment Variables
Create a `.env` file containing your database url. Run the following statement, replacing `{user}` and `{password}` with your MariaDB or MySQL database username and password.

```sh
echo "DATABASE_URL=mysql://{username}:{password}@localhost/" > .env
```

## Running

### Development
Run the dev script in the main repository.

```sh
./dev.sh
```

This script will build and run the backend, configuring it to listen on port `8081`. Then it will serve the frontend with trunk on port `8080`, proxying the api requests through to the backend. This allows for a reasonably fast hot reload when making changes to the frontend.

The dev script sets the environmental variable `DATA_SOURCE=federal_house` by default, which will attempt to a database of that name on a local MySQL or MariaDB server. The dev script also sets the `--dummy` command line flag by default, which tells the backend to return some hard-coded dummy data instead of running SQL queries. This is useful when you don't have the production data or need to test frontend changes quickly.

Connect to `127.0.0.1:8080` in your browser to use the tool, or connect to `localhost:8080` to run the Ontario version.

### Production
There is a simple prod script you can run that will compile both the frontend and backend in release mode and server the former's wasm binary via the latter.

```sh
./prod.sh
```

However, the best way to run this tool on production is in a docker container. A Dockerfile is provided in the repository, and you can build an image by running the following in the repository's root directory:

```sh
sudo docker build -t house-words ./
```

The image can be hosted via docker compose using a configuration file something like:

```yaml
---
services:
  :
    image: house-words:latest
    container_name: house-words
    environment:
      - DATABASE_URL=mysql://{username}:{password}@localhost/
      - DATA_SOURCE=federal_house
      - PORT=8080
    networks:
      - chunker_network
    ports:
      - 8080:8080
    restart: unless-stopped
```

Note that on a production server, this container should be run behind a reverse proxy. Since no volumes are required, you can create a dedicated user and group for the container and run it as that user and group with environmental variables PUID and PGID, and neither the user nor the group need read, write, or execute permissions *anywhere* on the server.

## Technical Info

### Overview
The repository is structured as a Rust workspace containing four packages. `backend` contains the [axum](https://crates.io/crates/axum/) web server and uses [diesel](https://crates.io/crates/diesel) to make queries to the database and pull the requested numbers. `frontend` is a [yew](https://crates.io/crates/yew) wasm app that makes use of the plotters library to render graphs on canvas elements. `db` holds diesel database schemas and join/group by rules for the backend, separated so as to be accessed both by the backend and as a library by independent translator repositories. `common` is a set of common types and serializable data models that the backend and frontend use to communicate.

Documentation is ongoing for this project. The Rust ecosystem has a secure foundation, but its novelty demands dedicated exploration. In the interest of sharing as much as I've learned as possible, and growing the sphere of open source public data accountability, I intend to continue to clarify component parts and add explanation to more complicated sections of the code.

### Backend
The backend for this project is relatively simple. Database calls are constructed using diesel, and the axum state holds onto a bb8 connection pool that allows each request handler to easily fire up a new connection. The most complicated part of the process is the reaper, which runs as an asynchronous loop waiting for messages from handlers. Each handler registers an active query with the reaper for the duration of its database request, and if a cancel message is received from the frontend for a particular user, all queries registered that user are cancelled.

### Frontend
The frontend is a single-page yew app. Pages are contained in the /pages folder, and everything more modular than that (including the navbar) is in the /components folder. The main interface page controls whether various charts are visible, and visible charts send queries to the backend when submits a new word. New word submissions trigger cancellation requests to the backend. The speech overlay (which appears when clicking on a particular breakdown bar or population point) sends its own requests and has its own cancellation call. The info page bundles its own data (for now) but is able to bring up a speech overlay for its demo graphs.

Charts are rendered using the CanvasBackend in plotters, which can render to a canvas element in yew via the `use_node_ref()` hook. Eventually, an SVG plotting library could open the door to much more sophisticated interactivity, but at this simple level (one hover function and one click function for each graph) the canvas element works well enough and keeps the DOM much smaller.

## Todo
- [x] Dropdown for graph selection
- [x] About me page 🫣
- [x] Transaction cancelling
- [x] Github readme
- [ ] Code comments and documentation
- [ ] Cancel on component destruction
- [ ] Ontario info page
- [ ] Ontario population scatter plot
- [ ] Time series selection
- [ ] Word or combinations
- [ ] Other provinces
- [ ] U.S. Congress?
