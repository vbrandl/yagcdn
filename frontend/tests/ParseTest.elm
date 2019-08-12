module ParseTest exposing
    ( emptyBranchGitHub
    , emptyRepoGitHub
    , emptyUserGitHub
    , invalidHostInUrlBitbucket
    , invalidHostInUrlGitHub
    , invalidHostInUrlGitLab
    , invalidUrlBitbucket
    , invalidUrlGitHub
    , invalidUrlGitLab
    , validHttpUrlBitbucket
    , validHttpUrlGitHub
    , validHttpUrlGitLab
    , validHttpsUrlBitbucket
    , validHttpsUrlGitHub
    , validHttpsUrlGitLab
    , validUrlWithoutProtocolBitbucket
    , validUrlWithoutProtocolGitHub
    , validUrlWithoutProtocolGitLab
    )

import Data exposing (Provider(..), Url)
import Expect
import Parse exposing (parseUrl)
import Test exposing (Test, test)


expectedUrl : Provider -> Url
expectedUrl prov =
    { prov = prov, user = "user", repo = "repo", gitref = "master", file = "README.md" }


expectedGitHubUrl : Url
expectedGitHubUrl =
    expectedUrl GitHub


expectedBitbucketUrl : Url
expectedBitbucketUrl =
    expectedUrl Bitbucket


expectedGitLabUrl : Url
expectedGitLabUrl =
    expectedUrl GitLab


validHttpsUrlGitHub : Test
validHttpsUrlGitHub =
    test "Parsing Valid HTTPS URL for GitHub"
        (\_ -> Expect.equal (Just expectedGitHubUrl) (parseUrl "https://GiThUb.CoM/user/repo/blob/master/README.md"))


validHttpUrlGitHub : Test
validHttpUrlGitHub =
    test "Parsing Valid HTTP URL for GitHub"
        (\_ -> Expect.equal (Just expectedGitHubUrl) (parseUrl "http://GiThUb.CoM/user/repo/blob/master/README.md"))


validUrlWithoutProtocolGitHub : Test
validUrlWithoutProtocolGitHub =
    test "Parsing Valid URL without Protocol for GitHub"
        (\_ -> Expect.equal (Just expectedGitHubUrl) (parseUrl "GiThUb.CoM/user/repo/blob/master/README.md"))


invalidUrlGitHub : Test
invalidUrlGitHub =
    test "Parsing Invalid URL for GitHub"
        (\_ -> Expect.equal Nothing (parseUrl "https://GiThUb.CoM/user"))


emptyUserGitHub : Test
emptyUserGitHub =
    test "Parsing Empty User URL for GitHub"
        (\_ -> Expect.equal Nothing (parseUrl "https://GiThUb.CoM//repo/blob/master/README.md"))


emptyRepoGitHub : Test
emptyRepoGitHub =
    test "Parsing Empty Repo URL for GitHub"
        (\_ -> Expect.equal Nothing (parseUrl "https://GiThUb.CoM/user//blob/master/README.md"))


emptyBranchGitHub : Test
emptyBranchGitHub =
    test "Parsing Empty Branch URL for GitHub"
        (\_ -> Expect.equal Nothing (parseUrl "https://GiThUb.CoM/user/repo/blob//README.md"))


invalidHostInUrlGitHub : Test
invalidHostInUrlGitHub =
    test "Parsing Invalid Host in URL for GitHub"
        (\_ -> Expect.equal Nothing (parseUrl "https://example.com/user/repo/blob/master/README.md"))


validHttpsUrlBitbucket : Test
validHttpsUrlBitbucket =
    test "Parsing Valid HTTPS URL for Bitbucket"
        (\_ -> Expect.equal (Just expectedBitbucketUrl) (parseUrl "https://bItBuCkEt.OrG/user/repo/src/master/README.md"))


validHttpUrlBitbucket : Test
validHttpUrlBitbucket =
    test "Parsing Valid HTTP URL for Bitbucket"
        (\_ -> Expect.equal (Just expectedBitbucketUrl) (parseUrl "http://BiTbUcKeT.oRg/user/repo/src/master/README.md"))


validUrlWithoutProtocolBitbucket : Test
validUrlWithoutProtocolBitbucket =
    test "Parsing Valid URL without Protocol for Bitbucket"
        (\_ -> Expect.equal (Just expectedBitbucketUrl) (parseUrl "bitbucket.org/user/repo/src/master/README.md"))


invalidUrlBitbucket : Test
invalidUrlBitbucket =
    test "Parsing Invalid URL for Bitbucket"
        (\_ -> Expect.equal Nothing (parseUrl "https://bitBucket.ORG/user"))


invalidHostInUrlBitbucket : Test
invalidHostInUrlBitbucket =
    test "Parsing Invalid Host in URL for Bitbucket"
        (\_ -> Expect.equal Nothing (parseUrl "https://example.com/user/repo/blob/src/README.md"))


validHttpsUrlGitLab : Test
validHttpsUrlGitLab =
    test "Parsing Valid HTTPS URL for GitLab"
        (\_ -> Expect.equal (Just expectedGitLabUrl) (parseUrl "https://GITLAB.CoM/user/repo/blob/master/README.md"))


validHttpUrlGitLab : Test
validHttpUrlGitLab =
    test "Parsing Valid HTTP URL for GitLab"
        (\_ -> Expect.equal (Just expectedGitLabUrl) (parseUrl "http://gItLaB.CoM/user/repo/blob/master/README.md"))


validUrlWithoutProtocolGitLab : Test
validUrlWithoutProtocolGitLab =
    test "Parsing Valid URL without Protocol for GitLab"
        (\_ -> Expect.equal (Just expectedGitLabUrl) (parseUrl "GiTlAb.CoM/user/repo/blob/master/README.md"))


invalidUrlGitLab : Test
invalidUrlGitLab =
    test "Parsing Invalid URL for GitLab"
        (\_ -> Expect.equal Nothing (parseUrl "https://gitLab.CoM/user"))


invalidHostInUrlGitLab : Test
invalidHostInUrlGitLab =
    test "Parsing Invalid Host in URL for GitLab"
        (\_ -> Expect.equal Nothing (parseUrl "https://example.com/user/repo/blob/master/README.md"))
