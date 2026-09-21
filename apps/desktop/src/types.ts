/** Shared Desktop shell types (shipctl JSON surfaces). */

export type CmdResult = {
  ok: boolean;
  code: number;
  stdout: string;
  stderr: string;
  shipctl: string;
  cancelled?: boolean;
};

export type StreamLine = {
  stream: "stdout" | "stderr" | "meta" | string;
  text: string;
};

export type ToolStatus = {
  found?: boolean;
  path?: string | null;
  version?: string | null;
};

export type Detected = {
  signet_toml?: boolean;
  package_json?: boolean;
  tauri?: boolean;
  wrangler?: boolean;
  vercel?: boolean;
  netlify?: boolean;
  github?: boolean;
  polar?: boolean;
  orbit_configured?: boolean;
  hints?: string[];
};

export type DoctorReport = {
  ok?: boolean;
  signet?: ToolStatus;
  orbit?: ToolStatus;
  notes?: string[];
  detected?: Detected;
};

export type PortalStep = {
  id?: string;
  provider?: string;
  kind?: string;
  title?: string;
  detail?: string;
  entry_url?: string | null;
  cli?: string[] | null;
  human?: boolean;
};

export type PortalPlan = {
  providers?: string[];
  steps?: PortalStep[];
  notes?: string[];
};

export type SecretsPlan = {
  hints?: Array<{
    provider?: string;
    name?: string;
    put_cli?: string[];
    entry_url?: string | null;
    detail?: string;
    source?: string;
  }>;
};

export type HumanSprint = {
  minutes_hint?: string;
  open_order?: string[];
  put_queue?: Array<{
    provider?: string;
    name?: string;
    put_cli?: string[];
    entry_url?: string | null;
    detail?: string;
  }>;
  checklist?: string[];
};

export type LaunchView = {
  current_index?: number;
  total?: number;
  done_count?: number;
  finished?: boolean;
  current?: {
    id?: string;
    title?: string;
    kind?: string;
    detail?: string;
    entry_url?: string | null;
    status?: string;
    verify_hint?: string | null;
    run?: string[] | null;
  } | null;
  steps?: Array<{
    id?: string;
    title?: string;
    status?: string;
    kind?: string;
  }>;
  notes?: string[];
};

export type PublishView = {
  mode?: string;
  intent?: string;
  current_index?: number;
  total?: number;
  done_count?: number;
  minutes_remaining?: number;
  minutes_total?: number;
  finished?: boolean;
  current?: {
    id?: string;
    title?: string;
    kind?: string;
    detail?: string;
    entry_url?: string | null;
    status?: string;
    minutes?: number;
    run?: string[] | null;
    desktop_view?: string | null;
  } | null;
  steps?: Array<{
    id?: string;
    title?: string;
    status?: string;
    kind?: string;
    minutes?: number;
  }>;
  notes?: string[];
};

export type PulseAction = {
  id?: string;
  label?: string;
  kind?: string;
  view?: string | null;
  cmd?: string[] | null;
};

export type ProjectPulse = {
  name?: string;
  kind?: string;
  git?: {
    is_repo?: boolean;
    branch?: string | null;
    dirty?: boolean;
    dirty_count?: number;
    committed?: boolean;
    ahead?: number | null;
    behind?: number | null;
    last_commit?: { hash?: string; subject?: string; when?: string | null } | null;
    notes?: string[];
  };
  publish?: {
    present?: boolean;
    finished?: boolean;
    current_index?: number;
    total?: number;
    done_count?: number;
    current_id?: string | null;
    current_title?: string | null;
    minutes_remaining?: number | null;
  };
  launch?: {
    present?: boolean;
    finished?: boolean;
    current_index?: number;
    total?: number;
    current_title?: string | null;
  };
  deploy?: {
    signal?: string;
    detail?: string;
    urls?: string[];
    last_run_ok?: boolean | null;
  };
  tools?: {
    signet_found?: boolean;
    orbit_found?: boolean;
    signet_version?: string | null;
    orbit_version?: string | null;
  };
  scopes_active?: string[];
  now?: {
    title?: string;
    detail?: string;
    primary?: PulseAction;
    actions?: PulseAction[];
  };
  notes?: string[];
};

export type ShipState = {
  project: string;
  has_ship_dir: boolean;
  studio: {
    sign_args?: string[];
    deploy_args?: string[];
    notes?: string[];
    detected?: Detected;
  } | null;
  last_run: {
    ok?: boolean;
    finished?: boolean;
    message?: string;
    steps?: Array<{ id?: string; ok?: boolean; exit_code?: number; detail?: string }>;
    finished_at?: string;
  } | null;
};

export type StudioMode = "general" | "advanced";
export type ShipIntent = "local" | "public";
export type ToastKind = "ok" | "err" | "info";

export type ScopePlan = {
  scopes?: Array<{
    id?: string;
    kind?: string;
    label?: string;
    relative?: string;
    provider?: string | null;
    signals?: string[];
  }>;
  active?: string[];
};

export type AssistPlan = {
  sign_path?: string;
  steps?: Array<{
    id?: string;
    title?: string;
    detail?: string;
    view?: string;
    ready?: boolean;
  }>;
};

export type EnvPortal = {
  actions?: Array<{
    id?: string;
    kind?: string;
    title?: string;
    detail?: string;
    entry_url?: string | null;
    put_cli?: string[] | null;
    provider?: string | null;
    name?: string | null;
  }>;
};

export type SignPortal = {
  recommended?: string;
  paths?: Array<{
    id?: string;
    kind?: string;
    title?: string;
    detail?: string;
    entry_url?: string | null;
    run?: string[] | null;
  }>;
};

export type IntegrationWizard = {
  id: string;
  group: "Payments" | "Email";
  title: string;
  blurb: string;
  provider?: string;
  openUrl: string;
  needsPublic: boolean;
  steps: string[];
};

export type CmdItem = {
  id: string;
  title: string;
  keywords: string;
  group: string;
  run: () => void;
};
