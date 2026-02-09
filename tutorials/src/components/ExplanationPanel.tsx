import { AnimatePresence, motion } from 'motion/react';
import Markdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';

interface ExplanationPanelProps {
  stepId: number;
  action: string;
  explanation: string;
}

export default function ExplanationPanel({ stepId, action, explanation }: ExplanationPanelProps) {
  return (
    <div className="explanation-panel">
      <AnimatePresence mode="wait">
        <motion.div
          key={stepId}
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -8 }}
          transition={{ duration: 0.25 }}
        >
          <h3 className="explanation-action">{action}</h3>
          <div className="explanation-body">
            <Markdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeHighlight]}>
              {explanation}
            </Markdown>
          </div>
        </motion.div>
      </AnimatePresence>
    </div>
  );
}
