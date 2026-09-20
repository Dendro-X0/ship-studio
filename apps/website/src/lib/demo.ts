/** Silent GIF shelf for /demo (docs/assets/demo/v0.1.0). */

export type DemoStage = {
  id: string;
  title: string;
  caption: string;
  src: string;
};

export const DEMO_VERSION = "v0.1.0";

export const DEMO_STAGES: DemoStage[] = [
  {
    id: "bind",
    title: "Bind project",
    caption: "Pick a local folder. Studio stays on your machine.",
    src: `/demo/${DEMO_VERSION}/01-bind.gif`,
  },
  {
    id: "open",
    title: "Publish · Open",
    caption: "Open the vendor door. Studio does not OAuth for you.",
    src: `/demo/${DEMO_VERSION}/02-open.gif`,
  },
  {
    id: "confirm",
    title: "Confirm → Next",
    caption: "You confirm each gate. Studio advances the spine.",
    src: `/demo/${DEMO_VERSION}/03-confirm-next.gif`,
  },
  {
    id: "preview",
    title: "Output Preview",
    caption: "Inspect local artifacts before the next Confirm.",
    src: `/demo/${DEMO_VERSION}/04-output-preview.gif`,
  },
];
