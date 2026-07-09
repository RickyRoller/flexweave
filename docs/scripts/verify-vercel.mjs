const { default: app } = await import("../.vercel/output/functions/__server.func/index.mjs");

const response = await app.fetch(new Request("http://localhost/docs"), {});

if (response.status !== 200) {
  console.error(`Expected /docs to return 200, received ${response.status}.`);
  console.error(await response.text());
  process.exit(1);
}

await response.body?.cancel();
console.log("Verified the Vercel bundle serves /docs.");
process.exit(0);
