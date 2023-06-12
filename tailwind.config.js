/** @type {import('tailwindcss').Config} */
module.exports = {
	content: {
		files: ["*.html", "./src/*.rs", "./src/**"]
	},
	theme: {
		extend: {
			height: {
				128: "36rem"
			},
			width: {
				128: "36rem"
			},
			fontFamily: {
				'h1': ['MagicSchoolTwo']
			}
		}
	},
	plugins: []
};
