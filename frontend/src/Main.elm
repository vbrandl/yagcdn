module Main exposing (Model, Msg(..), init, main, update, view)

import Browser
import Data exposing (Url, hostname, toHost, toUrl)
import Html exposing (Html, br, div, h1, input, table, td, text, tr)
import Html.Attributes exposing (disabled, placeholder, style, value)
import Html.Events exposing (onInput)
import Parse exposing (parseUrl)
import Ribbon exposing (ribbon)


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


renderUrl : Url -> Html msg
renderUrl { prov, user, repo, file } =
    div myStyle
        [ table myStyle
            [ tr myStyle
                [ td myStyle [ text "host" ]
                , td myStyle [ text (toHost prov) ]
                ]
            , tr []
                [ td myStyle [ text "user" ]
                , td myStyle [ text user ]
                ]
            , tr myStyle
                [ td myStyle [ text "repo" ]
                , td myStyle [ text repo ]
                ]
            , tr myStyle
                [ td myStyle [ text "file" ]
                , td myStyle [ text file ]
                ]
            ]
        ]


renderMUrl : Maybe Url -> Html msg
renderMUrl mUrl =
    mUrl
        |> Maybe.map renderUrl
        |> Maybe.withDefault (div myStyle [ text "Parse Error" ])


displayMUrl : Maybe Url -> String
displayMUrl mUrl =
    mUrl
        |> Maybe.map toUrl
        |> Maybe.withDefault ""


myStyle : List (Html.Attribute msg)
myStyle =
    [ style "width" "100%" ]


myStyle2 : List (Html.Attribute msg) -> List (Html.Attribute msg)
myStyle2 =
    List.append myStyle


view : Model -> Html Msg
view state =
    div []
        [ h1 []
            [ text "Gitache" ]
        , div
            myStyle
            [ ribbon
            , input (myStyle2 [ placeholder "URL to parse", value state.url, onInput UrlChange ]) []
            , div myStyle
                [ text "Parsed URL: "
                , br [] []
                , renderMUrl state.parsed
                ]
            , input (myStyle2 [ placeholder "https://" ++ hostname ++ "/<service>/<user>/<repo>/<gitref>/<file>", disabled True, value (displayMUrl state.parsed) ]) []
            ]
        ]


main : Program () Model Msg
main =
    Browser.sandbox { init = init, update = update, view = view }
