# transcoderexpress

Transcode media files in a directory to a target directory, will watch for new files and transcode them as they appear.

To run executable:

    $ cargo run -- -i /path/to/input -o /path/to/output

To run with Docker:

    $ docker run -v /path/to/input:/input -v /path/to/output:/output transcoderexpress
