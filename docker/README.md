Rustmas Docker Deployment
=========================

Prerequisites
-------------

1. Light positions captured and stored in a `lights.csv` file in the root of 
the project. You can use the provided example file [`lights.csv.example`](../lights.csv.example).

    ```
    cp lights.csv.example lights.csv
    ```

2. Rustmas configuration file. You can copy the provided 
[`Rustmas.example.yaml`](../Rustmas.example.yaml) file to `Rustmas.yaml` in 
the root directory and tweak the example settings.

    ```
    cp Rustmas.example.yaml Rustmas.yaml
    ```

3. An empty sqlite database file. The Web API runs appropriate migrations 
at startup, so there is no need to run any migrations manually. You can simply
copy the provided [`db.sqlite.example`](../db.sqlite.example) file in the root 
of the project and name it `db.sqlite`.

    ```
    cp db.sqlite.example db.sqlite
    ```


Getting and building the code
-----------------------------

* Install docker and docker-compose

* Navigate to the root directory of the repository and execute the build script:

    ```
    docker/build.sh
    ```

* Image for the Rustmas WebAPI (`localhost:rustmas-backend`) 
and Rustmas Web UI (`localhost:rustmas-frontend`) will be built

* To bring up the Rustmas Web App, execute the following command:

    ```
    docker compose -f docker/compose.yaml up
    ```
    
    Optionally you can use the `-d` flag to run it in the background. 
    
* The app will be running on [`localhost:8080`](http://localhost:8080)