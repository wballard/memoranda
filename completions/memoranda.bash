# Bash completion for memoranda

_memoranda() {
    local cur prev words cword
    _init_completion || return

    local commands="doctor serve help"
    local global_opts="--help -h --version -V"
    local doctor_opts="--verbose --auto-fix --help -h"

    case ${prev} in
        memoranda)
            COMPREPLY=($(compgen -W "${commands} ${global_opts}" -- ${cur}))
            return 0
            ;;
        doctor)
            COMPREPLY=($(compgen -W "${doctor_opts}" -- ${cur}))
            return 0
            ;;
        serve)
            COMPREPLY=($(compgen -W "--help -h" -- ${cur}))
            return 0
            ;;
        help)
            COMPREPLY=($(compgen -W "doctor serve" -- ${cur}))
            return 0
            ;;
        --help|-h|--version|-V)
            return 0
            ;;
        *)
            COMPREPLY=($(compgen -W "${global_opts}" -- ${cur}))
            return 0
            ;;
    esac
}

complete -F _memoranda memoranda