var i = 0;

setInterval(() => {
	switch (i) {
		case 0:
			document
				.getElementById('js_demonstration')
				.setAttribute('style', 'color: red');
			i++;
			break;
		case 1:
			document
				.getElementById('js_demonstration')
				.setAttribute('style', 'color: blue');
			i++;
			break;
		case 2:
			document
				.getElementById('js_demonstration')
				.setAttribute('style', 'color: green');
			i++;
			break;
		case 3:
			document
				.getElementById('js_demonstration')
				.setAttribute('style', 'color: black');
			i = 0;
			break;
	}
}, 500);
