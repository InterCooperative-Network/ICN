import React, { useState, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Progress } from '@/components/ui/progress'
import { AlertCircle, ChevronRight, Users, TrendingUp } from 'lucide-react'

type Proposal = {
  id: string
  title: string
  description: string
  status: 'active' | 'passed' | 'rejected'
  votesFor: number
  votesAgainst: number
  quorum: number
  createdBy: string
  endsAt: string
  totalVoters: number
  delegatedVotes: number
}

type VotingStats = {
  totalProposals: number
  activeProposals: number
  participationRate: number
  monthlyVotes: Array<{ month: string; votes: number }>
}

type ReputationUpdate = {
  did: string
  change: number
  newTotal: number
}

const GovernanceDashboard = () => {
  const [proposals, setProposals] = useState<Proposal[]>([])
  const [votingStats, setVotingStats] = useState<VotingStats>({
    totalProposals: 0,
    activeProposals: 0,
    participationRate: 0,
    monthlyVotes: []
  })
  const [selectedTab, setSelectedTab] = useState('active')
  const [loading, setLoading] = useState(true)
  const [reputationUpdates, setReputationUpdates] = useState<ReputationUpdate[]>([])

  useEffect(() => {
    // Mock data - replace with actual API calls
    const mockProposals: Proposal[] = [
      {
        id: '1',
        title: 'Community Resource Allocation Q3',
        description: 'Proposal to allocate community resources for Q3 projects',
        status: 'active',
        votesFor: 750,
        votesAgainst: 250,
        quorum: 1000,
        createdBy: 'did:icn:alice',
        endsAt: '2024-11-01',
        totalVoters: 1500,
        delegatedVotes: 200
      },
      {
        id: '2',
        title: 'New Cooperation Guidelines',
        description: 'Updated guidelines for inter-cooperative collaboration',
        status: 'passed',
        votesFor: 800,
        votesAgainst: 100,
        quorum: 1000,
        createdBy: 'did:icn:bob',
        endsAt: '2024-10-15',
        totalVoters: 1200,
        delegatedVotes: 150
      }
    ]

    const mockStats = {
      totalProposals: 45,
      activeProposals: 3,
      participationRate: 78.5,
      monthlyVotes: [
        { month: 'Jan', votes: 120 },
        { month: 'Feb', votes: 150 },
        { month: 'Mar', votes: 180 },
        { month: 'Apr', votes: 220 }
      ]
    }

    const mockReputationUpdates: ReputationUpdate[] = [
      {
        did: 'did:icn:alice',
        change: 10,
        newTotal: 110
      },
      {
        did: 'did:icn:bob',
        change: -5,
        newTotal: 95
      }
    ]

    setProposals(mockProposals)
    setVotingStats(mockStats)
    setReputationUpdates(mockReputationUpdates)
    setLoading(false)
  }, [])

  const calculateProgress = (votesFor: number, votesAgainst: number) => {
    const total = votesFor + votesAgainst
    return total > 0 ? (votesFor / total) * 100 : 0
  }

  const ProposalCard = ({ proposal }: { proposal: Proposal }) => (
    <Card className="p-4">
      <div className="flex justify-between items-start mb-4">
        <div>
          <h3 className="text-lg font-semibold">{proposal.title}</h3>
          <p className="text-sm text-gray-600">{proposal.description}</p>
        </div>
        <span className={`px-2 py-1 rounded text-sm ${
          proposal.status === 'active' ? 'bg-blue-100 text-blue-800' :
          proposal.status === 'passed' ? 'bg-green-100 text-green-800' :
          'bg-red-100 text-red-800'
        }`}>
          {proposal.status.charAt(0).toUpperCase() + proposal.status.slice(1)}
        </span>
      </div>

      <div className="space-y-2">
        <div className="flex justify-between text-sm">
          <span>Progress</span>
          <span>{calculateProgress(proposal.votesFor, proposal.votesAgainst).toFixed(1)}%</span>
        </div>
        <Progress 
          value={calculateProgress(proposal.votesFor, proposal.votesAgainst)} 
          className="h-2"
        />
        
        <div className="flex justify-between text-sm text-gray-600">
          <span>For: {proposal.votesFor}</span>
          <span>Against: {proposal.votesAgainst}</span>
        </div>

        <div className="flex justify-between items-center mt-4">
          <div className="text-sm text-gray-600">
            <p>Created by: {proposal.createdBy}</p>
            <p>Ends: {new Date(proposal.endsAt).toLocaleDateString()}</p>
          </div>
          {proposal.status === 'active' && (
            <Button className="space-x-2">
              <span>Vote Now</span>
              <ChevronRight className="h-4 w-4" />
            </Button>
          )}
        </div>
      </div>
    </Card>
  )

  const ReputationUpdateCard = ({ update }: { update: ReputationUpdate }) => (
    <Card className="p-4">
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
      </div>
    </Card>
  )

  return (
    <div className="container mx-auto p-4 space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-500">Active Proposals</p>
                <h3 className="text-2xl font-bold">{votingStats.activeProposals}</h3>
              </div>
              <AlertCircle className="h-8 w-8 text-blue-500" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-500">Participation Rate</p>
                <h3 className="text-2xl font-bold">{votingStats.participationRate}%</h3>
              </div>
              <Users className="h-8 w-8 text-green-500" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-500">Total Proposals</p>
                <h3 className="text-2xl font-bold">{votingStats.totalProposals}</h3>
              </div>
              <TrendingUp className="h-8 w-8 text-purple-500" />
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Voting Activity</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={votingStats.monthlyVotes}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="month" />
                <YAxis />
                <Tooltip />
                <Legend />
                <Line 
                  type="monotone" 
                  dataKey="votes" 
                  stroke="#8884d8"
                  strokeWidth={2}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Proposals</CardTitle>
        </CardHeader>
        <CardContent>
          <Tabs value={selectedTab} onValueChange={setSelectedTab}>
            <TabsList>
              <TabsTrigger value="active">Active</TabsTrigger>
              <TabsTrigger value="passed">Passed</TabsTrigger>
              <TabsTrigger value="rejected">Rejected</TabsTrigger>
            </TabsList>

            <TabsContent value="active" className="space-y-4">
              {proposals
                .filter(p => p.status === 'active')
                .map(proposal => (
                  <ProposalCard key={proposal.id} proposal={proposal} />
                ))}
            </TabsContent>

            <TabsContent value="passed" className="space-y-4">
              {proposals
                .filter(p => p.status === 'passed')
                .map(proposal => (
                  <ProposalCard key={proposal.id} proposal={proposal} />
                ))}
            </TabsContent>

            <TabsContent value="rejected" className="space-y-4">
              {proposals
                .filter(p => p.status === 'rejected')
                .map(proposal => (
                  <ProposalCard key={proposal.id} proposal={proposal} />
                ))}
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Reputation Updates</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {reputationUpdates.map((update, index) => (
            <ReputationUpdateCard key={index} update={update} />
          ))}
        </CardContent>
      </Card>

      <Alert>
        <AlertDescription>
          You currently have {proposals[0]?.delegatedVotes || 0} votes delegated to you. 
          Visit the delegation page to manage your voting power.
        </AlertDescription>
      </Alert>
    </div>
  )
}

export default GovernanceDashboard
