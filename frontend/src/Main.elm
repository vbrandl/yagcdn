module Main exposing
    ( Model
    , Msg(..)
    , init
    , main
    , update
    , view
    )

import Browser
import Data exposing (Url, hostname, repository, servicename, toUrl)
import Html
    exposing
        ( Html
        , a
        , article
        , h1
        , h2
        , input
        , li
        , nav
        , p
        , section
        , small
        , text
        , ul
        )
import Html.Attributes exposing (href, placeholder, readonly, style, value)
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
    section myStyle
        [ header
        , body state
        , footer
        ]


header : Html msg
header =
    Html.header []
        [ h1 myStyle [ text servicename ]
        , ribbon repository
        ]


footer : Html msg
footer =
    Html.footer myStyle
        [ nav []
            [ ul []
                [ li []
                    [ small []
                        [ text "Created by "
                        , a [ href "https://www.vbrandl.net/" ]
                            [ text "Valentin Brandl" ]
                        , text "."
                        ]
                    ]
                ]
            ]
        , nav []
            [ ul []
                [ li []
                    [ small []
                        [ a [ href repository ]
                            [ text "Repository" ]
                        ]
                    ]
                , li []
                    [ small []
                        [ a [ href "https://opensource.org/licenses/MIT" ]
                            [ text "MIT License" ]
                        ]
                    ]
                ]
            ]
        ]


body : Model -> Html Msg
body state =
    article myStyle
        [ input (myStyle2 [ placeholder "GitHub/GitLab/Bitbucket URL", value state.url, onInput UrlChange ])
            []
        , input (myStyle2 [ readonly True, value (displayMUrl state.parsed) ]) []
        , h2 myStyle
            [ text "Caching" ]
        , p myStyle
            [ text "Files are cached in the CDN for about one year" ]
        , h2 myStyle
            [ text "Invalidating the Cache" ]
        , p myStyle
            [ text "To delete a file from the cache, just request the file via HTTP "
            , code "DELETE"
            ]
        ]


code : String -> Html msg
code msg =
    Html.code [] [ text msg ]


main : Program () Model Msg
main =
    Browser.sandbox
        { init = init
        , update = update
        , view = view
        }
