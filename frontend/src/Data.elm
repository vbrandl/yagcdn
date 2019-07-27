module Data exposing (Provider(..), Url, hostname, pathSeparator, toHost, toUrl)


hostname : String
hostname =
    "https://gitcdn.tk/"


type Provider
    = GitHub
    | Bitbucket


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


pathSeparator : Provider -> String
pathSeparator prov =
    case prov of
        GitHub ->
            "blob"

        Bitbucket ->
            "src"


toUrl : Url -> String
toUrl { prov, user, repo, gitref, file } =
    hostname ++ toHost prov ++ String.join "/" [ user, repo, pathSeparator prov, gitref, file ]
