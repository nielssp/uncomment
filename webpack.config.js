const webpack = require('webpack');
const path = require('path');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const CopyPlugin = require("copy-webpack-plugin");

const languages = ['en', 'en-GB', 'en-US', 'da'];

module.exports = languages.map(language => {
    return {
        entry: {
            embed: './client/embed.ts',
            count: './client/count.ts',
        },
        output: {
            path: path.resolve(__dirname, 'dist/' + language),
        },
        module: {
            rules: [
                {
                    test: /\.ts$/,
                    use: 'ts-loader',
                    exclude: /node_modules/,
                },
                {
                    test: /\.(sa|sc|c)ss$/,
                    use: [
                        'style-loader',
                        'css-loader',
                        'sass-loader',
                    ],
                },
            ]
        },
        resolve: {
            extensions: ['.ts']
        },
        plugins: [
            new webpack.NormalModuleReplacementPlugin(
                /languages\/default$/,
                './languages/' + language
            ),
            new webpack.DefinePlugin({
                'LANGUAGE': JSON.stringify(language),
            }),
        ],
    };
}).concat([
    {
        entry: {
            dashboard: './client/dashboard/main.ts',
        },
        module: {
            rules: [
                {
                    test: /\.ts$/,
                    use: 'ts-loader',
                    exclude: /node_modules/,
                },
                {
                    test: /\.(sa|sc|c)ss$/,
                    use: [
                        MiniCssExtractPlugin.loader,
                        'css-loader',
                        'sass-loader',
                    ],
                },
                {
                    test: /\.html$/i,
                    loader: 'html-loader',
                },
            ],
        },
        resolve: {
            extensions: ['.ts']
        },
        plugins: [
            new MiniCssExtractPlugin(),
            new CopyPlugin({
                patterns: [
                    { from: './client/dashboard/static', to: '' },
                ]
            }),
        ],
    }
]);
