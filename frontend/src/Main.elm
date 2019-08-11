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
import Html exposing (Html, fieldset, input, label)
import Html.Attributes exposing (autofocus, for, id, placeholder, readonly, style, value)
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


myStyle : List (Html.Attribute msg) -> List (Html.Attribute msg)
myStyle =
    List.append [ style "width" "100%" ]


view : Model -> Html Msg
view state =
    fieldset []
        [ label [ for "url" ] []
        , input
            (myStyle
                [ id "url"
                , placeholder "GitHub/GitLab/Bitbucket URL"
                , value state.url
                , onInput UrlChange
                , autofocus True
                ]
            )
            []
        , label [ for "output" ] []
        , input (myStyle [ id "output", readonly True, value (displayMUrl state.parsed) ]) []
        ]


main : Program () Model Msg
main =
    Browser.sandbox
        { init = init
        , update = update
        , view = view
        }
