import { greet } from "wasm_app";
export function setupCounter(element: HTMLButtonElement) {
	let counter = 0;
	const setCounter = (count: number) => {
		counter = count;
		element.innerHTML = `count is ${counter}`;
	};
	element.addEventListener("click", () => {
		greet();
		setCounter(counter + 1);
	});
	setCounter(0);
}
