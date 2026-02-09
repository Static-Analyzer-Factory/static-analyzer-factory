import type { AlgorithmType, AlgorithmState, TraceDiff } from '../content/trace-types';
import PtaInspector from './state-inspectors/PtaInspector';
import IfdsInspector from './state-inspectors/IfdsInspector';
import IntervalInspector from './state-inspectors/IntervalInspector';
import MssaInspector from './state-inspectors/MssaInspector';
import DomInspector from './state-inspectors/DomInspector';
import CgInspector from './state-inspectors/CgInspector';
import KCfaInspector from './state-inspectors/KCfaInspector';
import SvfInspector from './state-inspectors/SvfInspector';
import type {
  PtaState, IfdsState, IntervalState, MssaState,
  KCfaState, SvfState, DomState, CgState,
} from '../content/trace-types';

interface StateInspectorProps {
  algorithm: AlgorithmType;
  state: AlgorithmState;
  diff: TraceDiff;
}

export default function StateInspector({ algorithm, state, diff }: StateInspectorProps) {
  switch (algorithm) {
    case 'andersen-pta':
      return <PtaInspector state={state as PtaState} diff={diff} />;
    case 'ifds-taint':
      return <IfdsInspector state={state as IfdsState} diff={diff} />;
    case 'interval-absint':
      return <IntervalInspector state={state as IntervalState} />;
    case 'memory-ssa':
      return <MssaInspector state={state as MssaState} />;
    case 'dominator-tree':
      return <DomInspector state={state as DomState} diff={diff} />;
    case 'callgraph-construction':
      return <CgInspector state={state as CgState} diff={diff} />;
    case 'kcfa-pta':
      return <KCfaInspector state={state as KCfaState} diff={diff} />;
    case 'sparse-vf':
      return <SvfInspector state={state as SvfState} diff={diff} />;
  }
}
