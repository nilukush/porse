# porse
Microservice in Rust to call the Pocket SDK created in Rust (`porus`)

# HOW TO build and run
* Create an empty folder `logs` if not present
* `docker-compose up --build -d`

Check the log files in `logs/porse.log` apart from `docker logs 88bdba95a9a6 -f --since 5h -n 1000`

# HOW TO stop or shutdown
* `docker-compose down`