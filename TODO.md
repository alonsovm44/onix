
# TODO

## cmd: init

Make init more robust and zero config by default 


## cmd: publish TODO

- fix CI pipeline not outputing the x86_64 for macOS

- fix the TUI always exiting with error 1 despite succesfully finishing the jobs

- make the polling logic more robust

- add a way to show and modify the version in config.yaml from the command line

## cmd: install TODO

- make the MVP of the install command
    - example: `onix install user@repo-v.X.Y.Z` for version specific install or `onix install user@repo` for latest release

    requirements:
        the installer must:
            - resolve the repo url from the input
            - resolve the host device OS and arch
            - download the specified binary for the correct arch and OS
            - extract the binary
            - place the binary in the correct location C:\onix or ~/.bin/onix
                - create the folder if not exists and set it to path if it is not in path
            - chmod +x for linux/macOS users

## other

remove .gendox folder from repo tracking, modify `g` to add a `g dmemoriae <folder/file>`

