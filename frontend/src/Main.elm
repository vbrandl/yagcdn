module Main exposing
    ( Model
    , Msg(..)
    , init
    , main
    , update
    , view
    )

import Browser
import Data exposing (Url, hostname, toUrl)
import Html exposing (Html, div, input)
import Html.Attributes exposing (placeholder, readonly, style, value)
import Html.Events exposing (onInput)
import Parse exposing (parseUrl)


type Msg
    = UrlChange String


type alias Model =
    { url : String
    , parsed : Maybe Url
    }


init : Model
init =
    { url = ""
    , parsed = Nothing
    }


update : Msg -> Model -> Model
update msg state =
    case msg of
        UrlChange newUrl ->
            { state | url = newUrl, parsed = parseUrl newUrl }


displayMUrl : Maybe Url -> String
displayMUrl mUrl =
    mUrl
        |> Maybe.map toUrl
        |> Maybe.withDefault (hostname ++ "<service>/<user>/<repo>/<gitref>/<file>")


myStyle : List (Html.Attribute msg)
myStyle =
    [ style "width" "100%" ]


myStyle2 : List (Html.Attribute msg) -> List (Html.Attribute msg)
myStyle2 =
    List.append myStyle


view : Model -> Html Msg
view state =
    div myStyle
        [ input (myStyle2 [ placeholder "GitHub/GitLab/Bitbucket URL", value state.url, onInput UrlChange ])
            []
        , input (myStyle2 [ readonly True, value (displayMUrl state.parsed) ]) []
        ]


main : Program () Model Msg
main =
    Browser.sandbox
        { init = init
        , update = update
        , view = view
        }
