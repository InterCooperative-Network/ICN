import React, { useEffect, useRef, useState } from 'react';
import { Card, CardHeader, CardContent, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';

type ConsensusStatus = 'Proposing' | 'Voting' | 'Finalizing' | 'Completed' | 'Failed';

interface ConsensusUpdate {
  round_number: number;
  status: ConsensusStatus;
  coordinator: string;
  votes_count: number;
}

interface BlockFinalized {
  block_number: number;
  transactions_count: number;
  timestamp: number;
}

interface ReputationUpdate {
  did: string;
  change: number;
  new_total: number;
}

interface ErrorMessage {
  code: string;
  message: string;
}

interface WebSocketMessage {
  type: 'ConsensusUpdate' | 'BlockFinalized' | 'ReputationUpdate' | 'Error';
  data: ConsensusUpdate | BlockFinalized | ReputationUpdate | ErrorMessage;
}

const ConsensusMonitor: React.FC = () => {
  const [messages, setMessages] = useState<WebSocketMessage[]>([]);
  const [connected, setConnected] = useState(false);
  const ws = useRef<WebSocket | null>(null);

  useEffect(() => {
    // Get DID from local storage or context
    const did = localStorage.getItem('userDid') || 'default-did';

    const connectWebSocket = () => {
      try {
        ws.current = new WebSocket('ws://localhost:8080/ws');
        
        ws.current.onopen = () => {
          setConnected(true);
          // Send DID in a message after connection
          ws.current?.send(JSON.stringify({ type: 'identify', did }));
        };

        ws.current.onmessage = (event) => {
          try {
            const message: WebSocketMessage = JSON.parse(event.data);
            setMessages((prev) => [...prev, message].slice(-5)); // Keep last 5 messages
          } catch (e) {
            console.error('Failed to parse WebSocket message:', e);
          }
        };

        ws.current.onclose = () => {
          setConnected(false);
          // Try to reconnect after 5 seconds
          setTimeout(connectWebSocket, 5000);
        };

        ws.current.onerror = (error) => {
          console.error('WebSocket error:', error);
          ws.current?.close();
        };
      } catch (error) {
        console.error('Failed to establish WebSocket connection:', error);
        setTimeout(connectWebSocket, 5000);
      }
    };

    connectWebSocket();

    return () => {
      if (ws.current) {
        ws.current.close();
      }
    };
  }, []);

  const renderConsensusUpdate = (data: ConsensusUpdate) => (
    <Card className="mb-4">
      <CardContent className="pt-6">
        <div className="flex justify-between items-start mb-2">
          <p className="font-medium">Consensus Round {data.round_number}</p>
          <Badge variant={
            data.status === 'Completed' ? 'default' :
            data.status === 'Failed' ? 'destructive' :
            'secondary'
          }>
            {data.status}
          </Badge>
        </div>
        <p className="text-sm text-gray-600">Coordinator: {data.coordinator}</p>
        <p className="text-sm text-gray-600">Votes: {data.votes_count}</p>
      </CardContent>
    </Card>
  );

  const renderBlockFinalized = (data: BlockFinalized) => (
    <Card className="mb-4">
      <CardContent className="pt-6">
        <div className="flex justify-between items-start mb-2">
          <p className="font-medium">New Block #{data.block_number}</p>
          <Badge>Finalized</Badge>
        </div>
        <p className="text-sm text-gray-600">Transactions: {data.transactions_count}</p>
        <p className="text-sm text-gray-600">
          Time: {new Date(data.timestamp * 1000).toLocaleString()}
        </p>
      </CardContent>
    </Card>
  );

  const renderReputationUpdate = (data: ReputationUpdate) => (
    <Card className="mb-4">
      <CardContent className="pt-6">
        <div className="flex justify-between items-start mb-2">
          <p className="font-medium">Reputation Change</p>
          <Badge variant={data.change > 0 ? 'default' : 'destructive'}>
            {data.change > 0 ? '+' : ''}{data.change}
          </Badge>
        </div>
        <p className="text-sm text-gray-600">DID: {data.did}</p>
        <p className="text-sm text-gray-600">New Total: {data.new_total}</p>
      </CardContent>
    </Card>
  );

  const renderMessage = (message: WebSocketMessage) => {
    switch (message.type) {
      case 'ConsensusUpdate':
        return renderConsensusUpdate(message.data as ConsensusUpdate);
      case 'BlockFinalized':
        return renderBlockFinalized(message.data as BlockFinalized);
      case 'ReputationUpdate':
        return renderReputationUpdate(message.data as ReputationUpdate);
      case 'Error':
        const errorData = message.data as ErrorMessage;
        return (
          <Alert variant="destructive" className="mb-4">
            <AlertDescription>
              Error {errorData.code}: {errorData.message}
            </AlertDescription>
          </Alert>
        );
      default:
        return null;
    }
  };

  return (
    <div className="space-y-4 p-4">
      <Card>
        <CardHeader>
          <div className="flex justify-between items-center">
            <CardTitle>Consensus Monitor</CardTitle>
            <Badge variant={connected ? "default" : "destructive"}>
              {connected ? 'Connected' : 'Disconnected'}
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {messages.map((msg, i) => (
              <div key={i}>{renderMessage(msg)}</div>
            ))}
            {messages.length === 0 && (
              <p className="text-gray-500 text-center py-4">
                No consensus updates yet
              </p>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default ConsensusMonitor;
