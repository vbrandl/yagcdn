module Data exposing (Provider(..), Url, hostname, pathSeparator, repository, servicename, toUrl)


repository : String
repository =
    "https://git.vbrandl.net/vbrandl/gitache"


servicename : String
servicename =
    "GitCDN"


hostname : String
hostname =
    "https://gitcdn.tk/"


type Provider
    = GitHub
    | Bitbucket
    | GitLab


type alias Url =
    { prov : Provider
    , user : String
    , repo : String
    , gitref : String
    , file : String
    }


toHost : Provider -> String
toHost prov =
    case prov of
        GitHub ->
            "github/"

        Bitbucket ->
            "bitbucket/"

        GitLab ->
            "gitlab/"


pathSeparator : Provider -> String
pathSeparator prov =
    case prov of
        GitHub ->
            "blob"

        Bitbucket ->
            "src"

        GitLab ->
            "blob"


toUrl : Url -> String
toUrl { prov, user, repo, gitref, file } =
    hostname ++ toHost prov ++ String.join "/" [ user, repo, gitref, file ]
