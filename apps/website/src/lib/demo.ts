/** Silent GIF shelf for /demo — live Harbor clips (docs/assets/demo/v0.2.1). */

export type DemoStage = {
  id: string;
  title: string;
  caption: string;
  src: string;
};

export const DEMO_VERSION = "v0.2.1";

/** T1–T7 live Harbor GIFs (T6 Sign Open · T7 Deploy Open). */
export const DEMO_STAGES: DemoStage[] = [
  {
    id: "bind",
    title: "Bind project",
    caption:
      "Pick a folder. Studio detects the ship layout — no cloud account.",
    src: `/demo/${DEMO_VERSION}/01-bind.gif`,
  },
  {
    id: "open",
    title: "Open surface",
    caption: "Open the right surface. Studio does not OAuth for you.",
    src: `/demo/${DEMO_VERSION}/02-open.gif`,
  },
  {
    id: "confirm",
    title: "Confirm · Continue",
    caption: "You attest human gates; Continue burns the rest.",
    src: `/demo/${DEMO_VERSION}/03-confirm-next.gif`,
  },
  {
    id: "preview",
    title: "Output Preview",
    caption: "Inspect local output when you need it.",
    src: `/demo/${DEMO_VERSION}/04-output-preview.gif`,
  },
  {
    id: "midflight",
    title: "Mid-flight Back",
    caption: "Mid-flight state survives Sign / Deployment detours.",
    src: `/demo/${DEMO_VERSION}/05-midflight.gif`,
  },
  {
    id: "sign-open",
    title: "Sign · Open",
    caption: "Studio opens the store / cert site — you finish there.",
    src: `/demo/${DEMO_VERSION}/06-sign-open.gif`,
  },
  {
    id: "deploy-open",
    title: "Deployment · Deploy",
    caption:
      "Deploy streams wrangler on this machine; Results show the live URL — Open dashboard to finish on Cloudflare.",
    src: `/demo/${DEMO_VERSION}/07-deploy-open.gif`,
  },
];
