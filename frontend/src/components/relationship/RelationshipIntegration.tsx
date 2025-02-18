import React, { useEffect, useState } from 'react';
import { Card, CardHeader, CardContent, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { HandHeart, MessageCircle, Users } from 'lucide-react';

// Types for our relationship data
interface Contribution {
  contributorDid: string;
  description: string;
  impactStory: string;
  date: string;
  context: string;
  witnesses: string[];
  feedback: Array<{
    fromDid: string;
    content: string;
    date: string;
    endorsementType: 'Verification' | 'Impact' | 'Character' | 'Skill';
  }>;
  tags: string[];
}

interface MutualAidInteraction {
  date: string;
  providerDid: string;
  receiverDid: string;
  description: string;
  impactStory?: string;
  reciprocityNotes?: string;
  tags: string[];
}

interface Relationship {
  memberOne: string;
  memberTwo: string;
  relationshipType: string;
  started: string;
  story: string;
  interactions: Array<{
    date: string;
    description: string;
    impact?: string;
    interactionType: string;
  }>;
  mutualEndorsements: Array<{
    fromDid: string;
    content: string;
    date: string;
    context: string;
    skills: string[];
  }>;
}

interface ReputationUpdate {
  did: string;
  change: number;
  newTotal: number;
  category: string; // Added category field
}

export default function RelationshipIntegration() {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [contributions, setContributions] = useState<Contribution[]>([]);
  const [relationships, setRelationships] = useState<Relationship[]>([]);
  const [mutualAid, setMutualAid] = useState<MutualAidInteraction[]>([]);
  const [reputationUpdates, setReputationUpdates] = useState<ReputationUpdate[]>([]);
  const [activeTab, setActiveTab] = useState('overview');

  useEffect(() => {
    // Load initial data
    const loadData = async () => {
      try {
        setLoading(true);
        
        // Example of data fetching - replace with actual API endpoints
        const response = await fetch('/api/relationships/current-user');
        const data = await response.json();
        
        setContributions(data.contributions);
        setRelationships(data.relationships);
        setMutualAid(data.mutualAid);
        setReputationUpdates(data.reputationUpdates);
        
        // Set up WebSocket connection for real-time updates
        const ws = new WebSocket('ws://localhost:8088/ws');
        
        ws.onmessage = (event) => {
          const update = JSON.parse(event.data);
          switch (update.type) {
            case 'contribution':
              setContributions(prev => [...prev, update.data]);
              break;
            case 'mutualAid':
              setMutualAid(prev => [...prev, update.data]);
              break;
            case 'relationship':
              setRelationships(prev => 
                prev.map(r => 
                  r.memberOne === update.data.memberOne ? update.data : r
                )
              );
              break;
            case 'reputationUpdate':
              setReputationUpdates(prev => [...prev, update.data]);
              break;
          }
        };

        return () => ws.close();

      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load relationships');
      } finally {
        setLoading(false);
      }
    };

    loadData();
  }, []);

  if (loading) {
    return <div className="p-4">Loading relationships...</div>;
  }

  if (error) {
    return (
      <Alert>
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="container mx-auto p-4 space-y-8">
      <Alert>
        <AlertDescription>
          Build stronger cooperative bonds through mutual aid, contributions, and shared stories.
        </AlertDescription>
      </Alert>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="overview">
            <Users className="h-4 w-4 mr-2" />
            Overview
          </TabsTrigger>
          <TabsTrigger value="mutual-aid">
            <HandHeart className="h-4 w-4 mr-2" />
            Mutual Aid
          </TabsTrigger>
          <TabsTrigger value="relationships">
            <MessageCircle className="h-4 w-4 mr-2" />
            Relationships
          </TabsTrigger>
          <TabsTrigger value="reputation-updates">
            <Users className="h-4 w-4 mr-2" />
            Reputation Updates
          </TabsTrigger>
        </TabsList>

        <TabsContent value="overview">
          <Card>
            <CardHeader>
              <CardTitle>Community Activity</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {/* Recent Activity Overview */}
                <div className="grid gap-4 md:grid-cols-2">
                  {/* Latest Contributions */}
                  <Card className="p-4">
                    <h3 className="font-medium mb-2">Recent Contributions</h3>
                    {contributions.slice(-3).map((contribution, i) => (
                      <div key={i} className="mb-2 text-sm">
                        <p className="font-medium">{contribution.description}</p>
                        <p className="text-gray-600">{contribution.impactStory}</p>
                      </div>
                    ))}
                  </Card>

                  {/* Latest Mutual Aid */}
                  <Card className="p-4">
                    <h3 className="font-medium mb-2">Recent Mutual Aid</h3>
                    {mutualAid.slice(-3).map((interaction, i) => (
                      <div key={i} className="mb-2 text-sm">
                        <p className="font-medium">{interaction.description}</p>
                        {interaction.impactStory && (
                          <p className="text-gray-600">{interaction.impactStory}</p>
                        )}
                      </div>
                    ))}
                  </Card>
                </div>

                {/* Active Relationships */}
                <div>
                  <h3 className="font-medium mb-2">Active Relationships</h3>
                  <div className="space-y-2">
                    {relationships.slice(-3).map((relationship, i) => (
                      <Card key={i} className="p-4">
                        <p className="font-medium">
                          With {relationship.memberTwo}
                        </p>
                        <p className="text-sm text-gray-600">{relationship.story}</p>
                        <div className="mt-2">
                          <span className="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded-full">
                            {relationship.relationshipType}
                          </span>
                        </div>
                      </Card>
                    ))}
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="mutual-aid">
          <Card>
            <CardHeader>
              <CardTitle>Mutual Aid Network</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {mutualAid.map((interaction, i) => (
                  <Card key={i} className="p-4">
                    <div className="flex items-start gap-4">
                      <HandHeart className="h-5 w-5 text-green-500 mt-1" />
                      <div>
                        <h3 className="font-medium">{interaction.description}</h3>
                        <p className="text-sm text-gray-500">
                          {new Date(interaction.date).toLocaleDateString()}
                        </p>
                        {interaction.impactStory && (
                          <p className="mt-2 text-gray-600">{interaction.impactStory}</p>
                        )}
                        {interaction.reciprocityNotes && (
                          <p className="mt-2 text-sm text-gray-600 italic">
                            {interaction.reciprocityNotes}
                          </p>
                        )}
                      </div>
                    </div>
                  </Card>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="relationships">
          <Card>
            <CardHeader>
              <CardTitle>Your Relationships</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {relationships.map((relationship, i) => (
                  <Card key={i} className="p-4">
                    <div className="space-y-4">
                      <div>
                        <h3 className="font-medium">
                          Relationship with {relationship.memberTwo}
                        </h3>
                        <p className="text-sm text-gray-500">
                          Since {new Date(relationship.started).toLocaleDateString()}
                        </p>
                      </div>
                      
                      <div className="bg-gray-50 p-4 rounded">
                        <h4 className="text-sm font-medium mb-1">Our Story</h4>
                        <p className="text-gray-600">{relationship.story}</p>
                      </div>

                      <div>
                        <h4 className="text-sm font-medium mb-2">Recent Interactions</h4>
                        {relationship.interactions.slice(-3).map((interaction, j) => (
                          <div key={j} className="mb-2 text-sm">
                            <p className="font-medium">{interaction.description}</p>
                            <p className="text-gray-600">
                              {new Date(interaction.date).toLocaleDateString()}
                            </p>
                          </div>
                        ))}
                      </div>
                    </div>
                  </Card>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="reputation-updates">
          <Card>
            <CardHeader>
              <CardTitle>Reputation Updates</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {reputationUpdates.map((update, i) => (
                  <Card key={i} className="p-4">
                    <div className="flex justify-between items-start mb-4">
                      <div>
                        <h3 className="text-lg font-semibold">Reputation Update</h3>
                        <p className="text-sm text-gray-600">DID: {update.did}</p>
                      </div>
                      <span className={`px-2 py-1 rounded text-sm ${
                        update.change > 0 ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                      }`}>
                        {update.change > 0 ? '+' : ''}{update.change}
                      </span>
                    </div>

                    <div className="space-y-2">
                      <div className="flex justify-between text-sm">
                        <span>New Total</span>
                        <span>{update.newTotal}</span>
                      </div>
                      <div className="flex justify-between text-sm">
                        <span>Category</span>
                        <span>{update.category}</span>
                      </div>
                    </div>
                  </Card>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
