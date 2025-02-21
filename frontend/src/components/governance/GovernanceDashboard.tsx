import React, { useState, useEffect, useRef, useCallback } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Progress } from '@/components/ui/progress'
import { AlertCircle, ChevronRight, Users, TrendingUp } from 'lucide-react'
import { Dialog, DialogOverlay, DialogContent } from '@reach/dialog'
import '@reach/dialog/styles.css'
import { FixedSizeList as List } from 'react-window'

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
  category: string // Added category field
}

type WebSocketMessage = {
  type: 'ProposalUpdate' | 'VoteUpdate' | 'ReputationUpdate'
  data: Proposal | ReputationUpdate
}

const ProposalRow = ({ index, style, data }) => {
  const proposal = data[index]
  return (
    <div style={style}>
      <ProposalCard proposal={proposal} />
    </div>
  )
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
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [newProposal, setNewProposal] = useState({ title: '', description: '' })
  const [formErrors, setFormErrors] = useState({ title: '', description: '' })
  const [isSubmitting, setIsSubmitting] = useState(false)
  const wsRef = useRef<WebSocket | null>(null)
  const listRef = useRef(null)
  const getListHeight = useCallback(() => {
    return Math.min(window.innerHeight * 0.6, proposals.length * 200)
  }, [proposals.length])

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
        newTotal: 110,
        category: 'governance' // Added category field
      },
      {
        did: 'did:icn:bob',
        change: -5,
        newTotal: 95,
        category: 'resource_sharing' // Added category field
      }
    ]

    setProposals(mockProposals)
    setVotingStats(mockStats)
    setReputationUpdates(mockReputationUpdates)
    setLoading(false)

    // WebSocket connection for real-time updates
    const connectWebSocket = () => {
      if (!wsRef.current) {
        try {
          wsRef.current = new WebSocket('ws://localhost:8080/ws')

          wsRef.current.onopen = () => {
            console.log('WebSocket connected')
          }

          wsRef.current.onmessage = (event) => {
            try {
              const message: WebSocketMessage = JSON.parse(event.data)
              handleWebSocketMessage(message)
            } catch (e) {
              console.error('Failed to parse WebSocket message:', e)
            }
          }

          wsRef.current.onclose = () => {
            console.log('WebSocket disconnected, attempting to reconnect...')
            wsRef.current = null
            setTimeout(connectWebSocket, 5000)
          }

          wsRef.current.onerror = (error) => {
            console.error('WebSocket error:', error)
            if (wsRef.current) {
              wsRef.current.close()
              wsRef.current = null
            }
          }
        } catch (error) {
          console.error('Failed to establish WebSocket connection:', error)
          setTimeout(connectWebSocket, 5000)
        }
      }
    }

    connectWebSocket()

    return () => {
      if (wsRef.current) {
        wsRef.current.close()
        wsRef.current = null
      }
    }
  }, [])

  const handleWebSocketMessage = (message: WebSocketMessage) => {
    switch (message.type) {
      case 'ProposalUpdate':
        setProposals((prevProposals) =>
          prevProposals.map((proposal) =>
            proposal.id === (message.data as Proposal).id ? (message.data as Proposal) : proposal
          )
        )
        break
      case 'VoteUpdate':
        setProposals((prevProposals) =>
          prevProposals.map((proposal) =>
            proposal.id === (message.data as Proposal).id ? (message.data as Proposal) : proposal
          )
        )
        break
      case 'ReputationUpdate':
        setReputationUpdates((prevUpdates) => [...prevUpdates, message.data as ReputationUpdate])
        break
      default:
        console.error('Unknown WebSocket message type:', message.type)
    }
  }

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
            <div className="space-x-2">
              <Button onClick={() => handleVote(proposal.id, true)}>Approve</Button>
              <Button onClick={() => handleVote(proposal.id, false)}>Reject</Button>
            </div>
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
        <div className="flex justify-between text-sm">
          <span>Category</span>
          <span>{update.category}</span>
        </div>
      </div>
    </Card>
  )

  const handleVote = async (proposalId: string, approve: boolean) => {
    try {
      const response = await fetch(`/api/governance/proposals/${proposalId}/vote`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ approve })
      })

      if (!response.ok) {
        throw new Error('Failed to cast vote')
      }

      // Update the proposal state with the new vote count
      setProposals(prevProposals =>
        prevProposals.map(proposal =>
          proposal.id === proposalId
            ? {
                ...proposal,
                votesFor: approve ? proposal.votesFor + 1 : proposal.votesFor,
                votesAgainst: !approve ? proposal.votesAgainst + 1 : proposal.votesAgainst
              }
            : proposal
        )
      )
    } catch (error) {
      console.error('Error voting on proposal:', error)
    }
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    const { name, value } = e.target
    setNewProposal(prevState => ({ ...prevState, [name]: value }))
  }

  const validateForm = () => {
    const errors = { title: '', description: '' }
    if (!newProposal.title) errors.title = 'Title is required'
    if (!newProposal.description) errors.description = 'Description is required'
    setFormErrors(errors)
    return !errors.title && !errors.description
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!validateForm()) return

    setIsSubmitting(true)
    try {
      const response = await fetch('/api/governance/proposals', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(newProposal)
      })

      if (!response.ok) {
        throw new Error('Failed to create proposal')
      }

      const createdProposal = await response.json()
      setProposals(prevProposals => [...prevProposals, createdProposal])
      setIsModalOpen(false)
      setNewProposal({ title: '', description: '' })
    } catch (error) {
      console.error('Error creating proposal:', error)
    } finally {
      setIsSubmitting(false)
    }
  }

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
          <Button onClick={() => setIsModalOpen(true)}>Create Proposal</Button>
          <Tabs value={selectedTab} onValueChange={setSelectedTab}>
            <TabsList>
              <TabsTrigger value="active">Active</TabsTrigger>
              <TabsTrigger value="passed">Passed</TabsTrigger>
              <TabsTrigger value="rejected">Rejected</TabsTrigger>
            </TabsList>

            <TabsContent value="active" className="space-y-4">
              <List
                ref={listRef}
                height={getListHeight()}
                itemCount={proposals.filter(p => p.status === 'active').length}
                itemSize={200}
                width="100%"
                itemData={proposals.filter(p => p.status === 'active')}
              >
                {ProposalRow}
              </List>
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

      <Dialog isOpen={isModalOpen} onDismiss={() => setIsModalOpen(false)} aria-label="Create Proposal">
        <DialogOverlay />
        <DialogContent>
          <h2 className="text-xl font-bold mb-4">Create Proposal</h2>
          <form onSubmit={handleSubmit}>
            <div className="mb-4">
              <label htmlFor="title" className="block text-sm font-medium text-gray-700">
                Title
              </label>
              <input
                type="text"
                id="title"
                name="title"
                value={newProposal.title}
                onChange={handleInputChange}
                className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
              />
              {formErrors.title && <p className="text-red-500 text-sm mt-1">{formErrors.title}</p>}
            </div>
            <div className="mb-4">
              <label htmlFor="description" className="block text-sm font-medium text-gray-700">
                Description
              </label>
              <textarea
                id="description"
                name="description"
                value={newProposal.description}
                onChange={handleInputChange}
                className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
              />
              {formErrors.description && <p className="text-red-500 text-sm mt-1">{formErrors.description}</p>}
            </div>
            <div className="flex justify-end">
              <Button type="button" onClick={() => setIsModalOpen(false)} className="mr-2">
                Cancel
              </Button>
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting ? 'Submitting...' : 'Submit'}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  )
}

export default GovernanceDashboard
