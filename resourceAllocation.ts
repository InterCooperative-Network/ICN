interface Member {
  id: string;
  reputation: number;
}

export function allocateResource(members: Member[]): Member[] {
  // Sort by reputation descending
  const prioritized = [...members].sort((a, b) => b.reputation - a.reputation);
  return prioritized;
}

export function decayReputation(members: Member[]): void {
  members.forEach(member => {
    member.reputation = Math.max(0, member.reputation * 0.98);
  });
}