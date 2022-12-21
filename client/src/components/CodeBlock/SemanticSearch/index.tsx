import { useCallback, useContext, useEffect, useState } from 'react';
import CodeBlockSearch from '../Search';
import Button from '../../Button';
import { ThumbsDown, ThumbsUp } from '../../../icons';
import { DeviceContext } from '../../../context/deviceContext';
import { getUpvote, saveUpvote } from '../../../services/api';
import useAppNavigation from '../../../hooks/useAppNavigation';

type Props = {
  answer: string;
  snippets: { code: string; path: string }[];
  onClick: () => void;
};
const SemanticSearch = ({ answer, snippets, onClick }: Props) => {
  const { deviceId } = useContext(DeviceContext);
  const { query } = useAppNavigation();
  const [isUpvoteLoading, setUpvoteLoading] = useState(true);
  const [isUpvote, setIsUpvote] = useState(false);
  const [isDownvote, setIsDownvote] = useState(false);

  // const results = [
  //   "listen(_: unknown, event: string, arg?: any): Event<any> {\n switch (event) {\n  default: throw new Error('no apples');\n }\n}",
  //   "listen(_: unknown, event: string, arg?: any): Event<any> {\n switch (event) {\n  default: throw new Error('no apples');\n }\n}",
  //   "listen(_: unknown, event: string, arg?: any): Event<any> {\n switch (event) {\n  default: throw new Error('no apples');\n }\n}",
  // ];

  useEffect(() => {
    setUpvoteLoading(true);
    setIsUpvote(false);
    setIsDownvote(false);
    getUpvote({ unique_id: deviceId, snippet_id: '1', query: query }).then(
      (resp) => {
        setUpvoteLoading(false);
        if (resp) {
          setIsUpvote(resp.is_upvote === true);
          setIsDownvote(resp.is_upvote === false);
        }
      },
    );
  }, [deviceId, query]);

  const handleUpvote = useCallback(
    (isUpvote: boolean) => {
      setIsUpvote(isUpvote);
      setIsDownvote(!isUpvote);
      return saveUpvote({
        unique_id: deviceId,
        is_upvote: isUpvote,
        query: query,
        snippet_id: '1',
        text: 'lorem ipsum',
      });
    },
    [deviceId, query],
  );
  return (
    <div className="flex flex-col">
      <div className="bg-gray-800 p-3 flex flex-row rounded-t relative">
        <span className="body-s pr-16">{answer}</span>
        {!isUpvoteLoading && (
          <div className="flex flex-row absolute top-3 right-3">
            <Button
              onlyIcon
              title="Upvote"
              variant={isUpvote ? 'secondary' : 'tertiary'}
              size="small"
              onClick={() => handleUpvote(true)}
            >
              <ThumbsUp />
            </Button>
            <Button
              onlyIcon
              title="Downvote"
              variant={isDownvote ? 'secondary' : 'tertiary'}
              size="small"
              onClick={() => handleUpvote(false)}
            >
              <ThumbsDown />
            </Button>
          </div>
        )}
      </div>
      {snippets.map((item, index) => (
        <span key={index} className={`${index ? 'mt-5' : ''}`}>
          <CodeBlockSearch
            snippets={[{ code: item.code, highlights: [] }]}
            language={'JavaScript'}
            filePath={item.path}
            branch={''}
            repoName={''}
            repoPath={''}
            hideMatchCounter
            hideDropdown
            onClick={onClick}
          />
        </span>
      ))}
    </div>
  );
};
export default SemanticSearch;
