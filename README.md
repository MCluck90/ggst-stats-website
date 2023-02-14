## GGST Science

Welcome to the Web part of the GGST Stats website. This repository hosts all the code used to display the data that we have. 

## Instructions to run
Clone the repository  
Edit example.env into .env  
Insert the required information. A typical database url looks something like `"postgresql://user:pass@localhost/databasename"`. Suggested IP and port is 0.0.0.0 8080  
Download the test database: <ggst-stats.net/sql/dump.sql>  
Go into `psql` and create a database called `ggststats`  
Go back, and run `psql ggststats < database.sql`   
`cargo run`. This should install all the dependencies and start up the server.


## Contribute

Contributing to this project is easy! Open up an issue in this repository, discuss the changes and you can get started. 

However, there are some details you should know before delving into this.

* Use the `is_cached` and `store_cache` functions to reduce required queries if the request is calling SQL Data. 

* All pages must have quality UX and great semantic HTML, to follow Web Accessibility standards.

* ### NO JAVASCRIPT

* Add your name to the CONTRIBUTORS.md file. By submitting your code to this repository, you submit the coder unde the AGPL license and agree that I'm allowed to use the code on my server, with zero restrictions or limitations, for no cost. 

## Code of Conduct

Be nice, be reasonable, and engage with others civilly. 
