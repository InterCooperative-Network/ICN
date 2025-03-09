import React, { useEffect, useRef } from 'react';
import * as d3 from 'd3';

interface Node {
  id: string;
  status: string;
  type: string;
}

interface Link {
  source: string;
  target: string;
}

interface GraphData {
  nodes: Node[];
  links: Link[];
}

export const NetworkGraph: React.FC = () => {
  const svgRef = useRef<SVGSVGElement>(null);
  const tooltipRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const fetchNetworkData = async () => {
      try {
        const response = await fetch('http://localhost:8080/api/network/topology');
        if (response.ok) {
          const data: GraphData = await response.json();
          renderGraph(data);
        }
      } catch (error) {
        console.error('Error fetching network topology:', error);
      }
    };

    fetchNetworkData();
    const interval = setInterval(fetchNetworkData, 10000);
    return () => clearInterval(interval);
  }, []);

  const renderGraph = (data: GraphData) => {
    if (!svgRef.current) return;

    const width = svgRef.current.clientWidth;
    const height = svgRef.current.clientHeight;

    // Clear existing graph
    d3.select(svgRef.current).selectAll('*').remove();

    const svg = d3.select(svgRef.current)
      .attr('width', width)
      .attr('height', height);

    const simulation = d3.forceSimulation()
      .force('link', d3.forceLink().id((d: any) => d.id))
      .force('charge', d3.forceManyBody().strength(-400))
      .force('center', d3.forceCenter(width / 2, height / 2));

    const getNodeColor = (status: string) => {
      switch (status) {
        case 'online': return '#22c55e';
        case 'offline': return '#ef4444';
        default: return '#eab308';
      }
    };

    // Create links
    const links = svg.append('g')
      .selectAll('line')
      .data(data.links)
      .enter()
      .append('line')
      .attr('stroke', '#cbd5e1')
      .attr('stroke-width', 1.5);

    // Create nodes
    const nodes = svg.append('g')
      .selectAll('circle')
      .data(data.nodes)
      .enter()
      .append('circle')
      .attr('r', 15)
      .attr('fill', d => getNodeColor(d.status))
      .call(d3.drag()
        .on('start', dragstarted)
        .on('drag', dragged)
        .on('end', dragended) as any);

    // Add node labels
    const labels = svg.append('g')
      .selectAll('text')
      .data(data.nodes)
      .enter()
      .append('text')
      .text(d => d.id)
      .attr('font-size', '12px')
      .attr('text-anchor', 'middle')
      .attr('dy', 30);

    // Add tooltips
    nodes.on('mouseover', function(event, d: any) {
      if (!tooltipRef.current) return;
      
      const tooltip = d3.select(tooltipRef.current)
        .style('display', 'block')
        .style('left', (event.pageX + 10) + 'px')
        .style('top', (event.pageY - 10) + 'px');

      tooltip.html(`
        <strong>${d.id}</strong><br>
        Type: ${d.type}<br>
        Status: ${d.status}
      `);
    })
    .on('mouseout', function() {
      if (!tooltipRef.current) return;
      d3.select(tooltipRef.current).style('display', 'none');
    });

    // Update positions on simulation tick
    simulation.nodes(data.nodes as any).on('tick', () => {
      links
        .attr('x1', (d: any) => d.source.x)
        .attr('y1', (d: any) => d.source.y)
        .attr('x2', (d: any) => d.target.x)
        .attr('y2', (d: any) => d.target.y);

      nodes
        .attr('cx', (d: any) => d.x)
        .attr('cy', (d: any) => d.y);

      labels
        .attr('x', (d: any) => d.x)
        .attr('y', (d: any) => d.y);
    });

    simulation.force<d3.ForceLink<any, any>>('link')?.links(data.links);

    // Drag functions
    function dragstarted(event: any) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      event.subject.fx = event.subject.x;
      event.subject.fy = event.subject.y;
    }

    function dragged(event: any) {
      event.subject.fx = event.x;
      event.subject.fy = event.y;
    }

    function dragended(event: any) {
      if (!event.active) simulation.alphaTarget(0);
      event.subject.fx = null;
      event.subject.fy = null;
    }
  };

  return (
    <div className="relative w-full h-full">
      <svg ref={svgRef} className="w-full h-full" />
      <div
        ref={tooltipRef}
        className="absolute hidden bg-black bg-opacity-75 text-white p-2 rounded text-sm pointer-events-none"
        style={{ zIndex: 1000 }}
      />
    </div>
  );
};