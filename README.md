# rrss2imap

[![Built with cargo-make](https://sagiegurari.github.io/cargo-make/assets/badges/cargo-make.svg)](https://sagiegurari.github.io/cargo-make)
[![Build Status](https://travis-ci.org/Riduidel/rrss2imap.svg?branch=master)](https://travis-ci.org/Riduidel/rrss2imap)

rrss2imap is a Rust reimplementation of the classical Python script [rss2imap](https://github.com/rcarmo/rss2imap)

Goals of this project include

* Having a reasonably performant implementation of rss2imap
* Learn Rust
* Explore parallel mechanism
* Maybe provide some kind of image embedding (DONE) with cache

## Getting Started

### Download rrss2imap

rrss2imap can be downloaded from [**releases page**](https://github.com/Riduidel/rrss2imap/releases). 
If there is no release for your platform, you can fill an issue ... or if you know Travis, you can even add your platform to `.travis.yml`.

### As a user

<!-- cargo-sync-readme start -->

Application transforming rss feeds into email by directly pushing the entries into IMP folders.
This application is an adaption of the rss2imap Python script to Rust.

#### How to use ?

The simplest way to understand what to do is just to run `rrss2imap --help`

It should output something like

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information
    
    SUBCOMMANDS:
        add       Adds a new feed given its url
        delete    Delete the given feed
        email     Changes email address used in feed file to be the given one
        export    Export subscriptions as opml file
        help      Prints this message or the help of the given subcommand(s)
        import    import the given opml file into subscriptions
        list      List all feeds configured
        new       Creates a new feedfile with the given email address
        reset     Reset feedfile (in other words, remove everything)
        run       Run feed parsing and transformation

Which give you a glimpse of what will happen

Each of these commands also provide some help, when run with the same `--help` flag.

The important operations to memorize are obviously

#### `rrss2imap new`

Creates a new `config.json` file. At init time, the config file will only contains `settings` element 
with the email address set. You **have** to set 

* the used imap server
** with user login and password
** and security settings (secure should contain `{"Yes": secure port}` for imap/s
or `{"No": unsecure port}` for simple imap)
* the default config
** folder will be the full path to an imap folder where entries will fall in
** email will be the recipient email address (which may not be yours for easier filtering)
** Base64 image inlining
* feeds is the list of all rss feeds that can be added

#### `rrss2imap add`

This command will add a new feed to your config. You can directly set here the email recipient as well as the folder
(but not the base64 image inlining parameter)

#### `rrss2imap run`

THis is the main command. It will

1. get all rss/atom feed contents
2. List all new entries in these feeds
3. Transform these entries into valid email messages
4. Push these mail messages directly on IMAP server

#### `rrss2imap list`

Displays a list of the rss feeds. Here is an example

```
0 : http://tontof.net/?rss (to: Nicolas Delsaux <nicolas.delsaux@gmx.fr> (default)) RSS/rrss2imap (default)
1 : https://www.brothers-brick.com/feed/ (to: Nicolas Delsaux <nicolas.delsaux@gmx.fr> (default)) RSS/rrss2imap (default)
2 : https://nicolas-delsaux.hd.free.fr/rss-bridge/?action=display&bridge=LesJoiesDuCode&format=AtomFormat (to: Nicolas Delsaux <nicolas.delsaux@gmx.fr> (default)) RSS/rrss2imap (default)
```

Please notice that each entry has an associated number, which is the one to enter when running `rrss2imap delete <NUMBER>`

#### `config.json` format

A typical feedfile will look like this

```json
    {
      "settings": {
        "email": {
          "server": "the imap server of your mail provider",
          "user": "your imap user name",
          "password": "your imap user password",
          "secure": {
            "Yes": 993 // Set to "Yes": port for imaps or "No": port for unsecure imap
          }
        },
        // This config is to be used for all feeds
        "config": {
            // This is the email address written in each mail sent. It can be different from the email user
            "email": "Nicolas Delsaux <nicolas.delsaux@gmx.fr>",
            // This is the imap folder in which mails will be written
            "folder": "RSS/rrss2imap"
            // Setting this to true will force rrss2imap to transform all images into
            // base64. This prevents images from beind downloaded (and is really cool when reading feeds from a smartphone)
            // But largely increase each mail size (which can be quite bothering)
            "inline_image_as_data": true
        }
      },
      "feeds": [
        {
          "url": "http://tontof.net/?rss",
          // This last updated is updated for each entry and should be enough to have rss items correctly read
          "last_updated": "2019-05-04T16:53:15",
          "config": {
              // each config element can be overwritten at the feed level
          }
        },
```
    

<!-- cargo-sync-readme end -->

### As a developer
* clone this repository
* run `cargo run`

#### Prerequisites

You need a complete rust build chain.  On Linux, you need to install the `libdbus-1-dev` and `pkg-config` packages installed while building, and the `libdbus-1-3` package installed while running.

To perform a release, you'll also need

* [cargo make](https://github.com/sagiegurari/cargo-make#usage-predefined-flows)
* [cargo release](https://github.com/sunng87/cargo-release)
* [git journal](https://github.com/saschagrunert/git-journal)

#### Installing

A step by step series of examples that tell you how to get a development env running

Say what the step will be

```
Give the example
```

And repeat

```
until finished
```

End with an example of getting some data out of the system or using it for a little demo

### Running the tests

Explain how to run the automated tests for this system

#### Break down into end to end tests

Explain what these tests test and why

```
Give an example
```

#### And coding style tests

Explain what these tests test and why

```
Give an example
```

## Deployment

Add additional notes about how to deploy this on a live system

## Built With

Take a look at Cargo dependencies

## Contributing

Please read [CONTRIBUTING.md](https://gist.github.com/PurpleBooth/b24679402957c63ec426) for details on our code of conduct, and the process for submitting pull requests to us.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/your/project/tags). 

## Authors

* **Nicolas Delsaux** - *Initial work* - [Riduidel](https://github.com/Riduidel)

See also the list of [contributors](https://github.com/Riduidel/rrss2imap/contributors) who participated in this project.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details

## Acknowledgments

* [Rui Carno](https://github.com/rcarmo) for Python implementation of [rss2imap](https://github.com/rcarmo/rss2imap)
* [Aaron Swartz](https://en.wikipedia.org/wiki/Aaron_Swartz) for [RSS](https://en.wikipedia.org/wiki/RSS) (and [rss2email](https://github.com/rss2email/rss2email))

