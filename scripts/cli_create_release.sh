npm -g install github-app-installation-token

echo ${env.APP_ID}
echo "${env.GH_APP_PRIVATE_KEY}" > public-release.pem
github-app-installation-token --appId ${env.APP_ID} --installationId ${env.INSTALLATION_ID} --privateKeyLocation public-release.2022-09-19.private-key.pem >> .my_token
rm public-release.pem

gh auth login --with-token < .api_key
gh auth status
rm .api_key

gh release create v0.0.0 my.zip --notes "bugfix release" --draft -R https://github.com/pontem-network/eth2move-samples
echo "Work is done"
