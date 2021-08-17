<h1><img src="client/dashboard/static/logo.svg" height=50 alt="Uncomment"/></h1>

Uncomment is a commenting system for blogs and static sites written in Rust and TypeScript.

**Work in progress**

## Features

* Lightweight
* Nested comments
* Markdown
* Moderation
* SQLite or PostgreSQL
* Import from Disqus

## Todo

* Optional user authentication
* Optional third party authentication
* Rule system for automatic moderation
* Akismet (maybe)

## Usage

Use docker to pull the latest development version of Uncomment for use with an SQLite database:

```
docker pull nielssp/uncomment:sqlite
```

Create a new envioronment file with at least the following settings:

```
UNCOMMENT_HOST=https://your-website.com,https://uncomment.your-website.com
UNCOMMENT_SECRET_KEY=<secret key used (a long with a random salt) for hashing password>
UNCOMMENT_DEFAULT_ADMIN_USERNAME=admin
UNCOMMENT_DEFAULT_ADMIN_PASSWORD=<password for first login>
```

You can generate a random secret key with `openssl rand -base64 20`.

Launch the Uncomment server:

```
docker run --rm --name uncomment -p 8080:8080 --env-file <your-env-file> -v <path-to-db-dir>:/db nielssp/uncomment:master
````

Add the following to your website:

```html
<div id="comments"></div>
<script data-uncomment
    data-uncomment-target="#comments"
    src="https://uncomment.your-website.com/en-GB/embed.js"></script>
```

### PostgreSQL

To use PostgreSQL, pull the postgres tag instead:

```
docker pull nielssp/uncomment:postgres
```

Add a connection string to the environment file:

```
UNCOMMENT_DATABASE=postgresql://username:password@hostname:port/dbname
```

When using a local PostgreSQL database it may make sense to add `--network=host` to the `docker run` command.

## Server Configuration

Uncomment is configured via environment variables.

* `UNCOMMENT_LISTEN=127.0.0.1:5000` &ndash; hostname and port to listen to
* `UNCOMMENT_HOST` &ndash; comma-separated list of websites that will be accessing Uncomment
* `UNCOMMENT_DATABASE=sqlite:data.db` &ndash; database connection string
* `UNCOMMENT_SECRET_KEY` &ndash; secret key used as part of Argon2 hash used for password hashing
* `UNCOMMENT_ARGON2_ITERATIONS=192` &ndash; number of Argon2 iterations to use, more iterations means more secure hash but slower login
* `UNCOMMENT_ARGON2_MEMORY_SIZE=4096`
* `UNCOMMENT_RATE_LIMIT=10` &ndash; maximum number of comments allowed from a single IP address within the time period specified by `UNCOMMENT_RATE_LIMIT_INTERVAL`
* `UNCOMMENT_RATE_LIMIT_INTERVAL=10` &ndash; minutes
* `UNCOMMENT_AUTO_THREADS=true` &ndash; automatically create threads. If disabled you must manually create threads in the dashboard.
* `UNCOMMENT_THREAD_URL` &ndash; thread URL used to validate new threads, use `%name%` as the thread name placeholder, e.g. `UNCOMMENT_THREAD_URL=https://myblog.com/blog/%name%`
* `UNCOMMENT_REQUIRE_NAME=false` &ndash; whether a name is required for posting comments, client should be configured to match
* `UNCOMMENT_REQUIRE_EMAIL=false` &ndash; whether an email is required for posting comments, client should be configured to match
* `UNCOMMENT_MODERATE_ALL=false` &ndash; whether all new comments should be marked as pending
* `UNCOMMENT_MAX_DEPTH=6` &ndash; maximum level of nesting allowed, cannot be higher than 6. 0 means that the comment list is completely flat and all replies are added to the end of the list.
* `UNCOMMENT_DEFAULT_ADMIN_USERNAME` &ndash; default username of admin user created automatically when no admin users exist
* `UNCOMMENT_DEFAULT_ADMIN_PASSWORD` &ndash; default password of admin user created automatically when no admin users exist

## Client Configuration

* `data-uncomment-target` &ndash; selector for the container element used to contain the comments
* `data-uncomment-id` &ndash; thread name to use instead of `location.pathname`
* `data-uncomment-relative-dates` &ndash; display relative dates like "4 weeks ago", enabled by default
* `data-uncomment-newest-first` &ndash; whether to display comments sorted chronologically in descending order instead of ascending order
* `data-uncomment-require-name` &ndash; whether a name is required for posting comments, server should be configured to match
* `data-uncomment-require-email` &ndash; whether an email is required for posting comments, server should be configured to match
* `data-uncomment-click-to-load` &ndash; whether to present the user with a button for loading the comments instead of automatically loading them when the page loads
