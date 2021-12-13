import * as wasm from "../pkg"

const constants = new wasm.LinksConstants();
let links = new wasm.UnitedLinks(constants);

let link = links.create();
link = links.update(link, link, link);

console.log("The number of links in the data store is " + links.count() + ".");
console.log("Data store contents:");

let cont = links.constants._continue;
let any = links.constants.any;
links.each(function (link) {
    console.log(link);
    return cont;
}, new wasm.Link(any, any, any));

links.delete(link);
