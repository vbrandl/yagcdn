module Parse exposing (parseUrl)

import Data exposing (Provider(..), Url, pathSeparator)


parseUrl : String -> Maybe Url
parseUrl url =
    stripProtocol url
        |> splitProvider
        |> Maybe.andThen splitOfHead
        |> Maybe.andThen splitOfHead
        |> Maybe.andThen splitOfHead
        |> Maybe.andThen splitOfHead
        |> Maybe.andThen
            (\( ( ( ( ( prov, user ), repo ), separator ), gitref ), file ) ->
                if List.isEmpty file || (separator /= pathSeparator prov) then
                    Nothing

                else
                    Just
                        { prov = prov
                        , user = user
                        , repo = repo
                        , gitref = gitref
                        , file = String.join "/" file
                        }
            )


splitOfHead : ( a, List b ) -> Maybe ( ( a, b ), List b )
splitOfHead ( head, tail ) =
    splitPart tail
        |> Maybe.map (\( h, t ) -> ( ( head, h ), t ))


stripProtocol : String -> String
stripProtocol url =
    let
        index =
            String.indexes "://" url
                |> List.head
                |> Maybe.withDefault -3
    in
    String.dropLeft (index + 3) url


parseProvider : String -> Maybe Provider
parseProvider prov =
    case String.toLower prov of
        "github.com" ->
            Just GitHub

        "bitbucket.org" ->
            Just Bitbucket

        _ ->
            Nothing


splitProvider : String -> Maybe ( Provider, List String )
splitProvider url =
    let
        split =
            String.split "/" url

        parts =
            splitPart split
    in
    parts
        |> Maybe.andThen
            (\( head, tail ) ->
                parseProvider head
                    |> Maybe.map (\prov -> ( prov, tail ))
            )


splitPart : List a -> Maybe ( a, List a )
splitPart parts =
    let
        head =
            List.head parts

        tail =
            List.tail parts
    in
    head
        |> Maybe.andThen (\h -> Maybe.map (\t -> ( h, t )) tail)
