var i = 0;

setInterval(() => {
	if (document.getElementById('js_demonstration')) {
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
	}
}, 500);

function showMessage(message) {
	// console.log('show');
	// console.log(message);
	document.querySelector('#message').innerHTML = message;
}
async function submitMessage(message) {
	const response = await fetch('/message', {
		method: 'POST', // *GET, POST, PUT, DELETE, etc.
		mode: 'cors', // no-cors, *cors, same-origin
		cache: 'no-cache', // *default, no-cache, reload, force-cache, only-if-cached
		credentials: 'same-origin', // include, *same-origin, omit
		headers: {
			'Content-Type': 'application/json'
			// 'Content-Type': 'application/x-www-form-urlencoded',
		},
		redirect: 'follow', // manual, *follow, error
		referrerPolicy: 'no-referrer', // no-referrer, *no-referrer-when-downgrade, origin, origin-when-cross-origin, same-origin, strict-origin, strict-origin-when-cross-origin, unsafe-url
		body: JSON.stringify({ body: message }) // body data type must match "Content-Type" header
	});
	if (!response.ok) {
		console.log('the submit message request failed');
		return false;
	} else {
		let resp = await response.text();
		return true;
	}
}

async function getMessage() {
	fetch('/message')
		.then(async (response) => {
			if (!response.ok) {
				console.log('the get message request failed');
			}
			const res = await response.text();
			return res;
		})
		.then((message) => showMessage(message))
		.catch((err) => console.error(`Fetch problem: ${err.message}`));
}

async function newMessage() {
	const message = document.querySelector('#new-message').value;
	if (message != '') {
		let resp = await submitMessage(message);
		if (resp) {
			getMessage();
		}
	}
}
getMessage();
