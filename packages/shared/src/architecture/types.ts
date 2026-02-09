export interface ArchitectureData {
  meta: { title: string; version: string };
  actors: Actor[];
  layers: Layer[];
  connections: Connection[];
}

export interface Actor {
  id: string;
  label: string;
  icon: 'robot' | 'user';
}

export interface NodeGroup {
  id: string;
  label: string;
  hub?: string;
  children: string[];
}

export interface Layer {
  id: string;
  label: string;
  color: string;
  groups?: NodeGroup[];
  nodes: ArchNode[];
}

export interface ArchNode {
  id: string;
  label: string;
  desc: string;
  crate?: string;
  path?: string;
  tags?: string[];
}

export interface Connection {
  from: string;
  to: string;
  type: 'uses' | 'data' | 'compiles-to' | 'deploys' | 'planned';
  label?: string;
}
