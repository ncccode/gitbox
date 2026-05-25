import type { CommitSummary } from "../types/gitbox";

const graphLaneWidth = 14;
const graphLaneInset = 10;
const graphRowHeight = 30;
const graphRowMid = graphRowHeight / 2;
const graphMaxVisibleLanes = 6;
const graphPalette = ["#b89445", "#8e63c8", "#4f9d76", "#4f82c9", "#c86d56", "#70a6a1"];

type LogGraphPath = {
  key: string;
  d: string;
  color: string;
};

type LogGraphActiveLane = {
  oid: string;
  color: string;
};

export type LogGraphRow = {
  item: CommitSummary;
  paths: LogGraphPath[];
  laneIndex: number;
  color: string;
  nodeLeft: number;
  graphWidth: number;
  hasMerge: boolean;
};

export function buildLogGraphRows(commits: CommitSummary[]): LogGraphRow[] {
  const rows: LogGraphRow[] = [];
  let lanes: LogGraphActiveLane[] = [];
  let colorCursor = 0;

  const nextColor = () => {
    const color = graphPalette[colorCursor % graphPalette.length];
    colorCursor += 1;
    return color;
  };

  for (const item of commits) {
    let laneIndex = lanes.findIndex((lane) => lane.oid === item.oid);
    if (laneIndex === -1) {
      laneIndex = Math.min(lanes.length, graphMaxVisibleLanes - 1);
      lanes.splice(laneIndex, 0, { oid: item.oid, color: nextColor() });
    }

    const topLanes = lanes.map((lane) => ({ ...lane }));
    const currentLane = topLanes[laneIndex];
    const nextLanes = topLanes.filter((_lane, index) => index !== laneIndex);
    const [firstParent, ...mergeParents] = item.parents;

    if (firstParent) {
      const existingParentIndex = nextLanes.findIndex((lane) => lane.oid === firstParent);
      if (existingParentIndex === -1) {
        nextLanes.splice(Math.min(laneIndex, nextLanes.length), 0, {
          oid: firstParent,
          color: currentLane.color,
        });
      }
    }

    mergeParents.forEach((parent, parentIndex) => {
      if (nextLanes.some((lane) => lane.oid === parent)) return;
      nextLanes.splice(Math.min(laneIndex + parentIndex + 1, nextLanes.length), 0, {
        oid: parent,
        color: nextColor(),
      });
    });

    const visibleTopLanes = topLanes.slice(0, graphMaxVisibleLanes);
    const visibleNextLanes = nextLanes.slice(0, graphMaxVisibleLanes);
    const bottomIndexByOid = new Map(visibleNextLanes.map((lane, index) => [lane.oid, index]));
    const paths: LogGraphPath[] = [];

    visibleTopLanes.forEach((lane, index) => {
      if (index === laneIndex && lane.oid === item.oid) return;
      const x = graphLaneX(index);
      const bottomIndex = bottomIndexByOid.get(lane.oid);
      const d =
        bottomIndex === undefined
          ? graphPathBetween(x, 0, x, graphRowMid)
          : graphPathBetween(x, 0, graphLaneX(bottomIndex), graphRowHeight);
      paths.push({
        key: `${item.oid}-lane-${lane.oid}-${index}`,
        d,
        color: lane.color,
      });
    });

    if (laneIndex < graphMaxVisibleLanes) {
      const nodeX = graphLaneX(laneIndex);
      paths.push({
        key: `${item.oid}-node-in`,
        d: graphPathBetween(nodeX, 0, nodeX, graphRowMid),
        color: currentLane.color,
      });

      item.parents.forEach((parent, parentIndex) => {
        const bottomIndex = bottomIndexByOid.get(parent);
        if (bottomIndex === undefined) return;
        const parentLane = visibleNextLanes[bottomIndex];
        paths.push({
          key: `${item.oid}-parent-${parent}-${parentIndex}`,
          d: graphPathBetween(nodeX, graphRowMid, graphLaneX(bottomIndex), graphRowHeight),
          color: parentIndex === 0 ? currentLane.color : parentLane.color,
        });
      });
    }

    const laneCount = Math.max(
      1,
      Math.min(graphMaxVisibleLanes, Math.max(visibleTopLanes.length, visibleNextLanes.length, laneIndex + 1)),
    );

    rows.push({
      item,
      paths,
      laneIndex,
      color: currentLane.color,
      nodeLeft: graphLaneX(laneIndex),
      graphWidth: Math.max(42, graphLaneX(laneCount - 1) + 12),
      hasMerge: item.parents.length > 1,
    });

    lanes = nextLanes.slice(0, graphMaxVisibleLanes);
  }

  return rows;
}

export function logGraphStyle(row: LogGraphRow) {
  return { width: `${row.graphWidth}px` };
}

export function logGraphViewBox(row: LogGraphRow) {
  return `0 0 ${row.graphWidth} ${graphRowHeight}`;
}

export function logNodeStyle(row: LogGraphRow) {
  return {
    left: `${row.nodeLeft}px`,
    backgroundColor: row.color,
  };
}

function graphLaneX(index: number) {
  return graphLaneInset + index * graphLaneWidth;
}

function graphPathBetween(x1: number, y1: number, x2: number, y2: number) {
  if (x1 === x2) {
    return `M ${x1} ${y1} L ${x2} ${y2}`;
  }

  const controlOffset = Math.max(5, Math.min(9, Math.abs(y2 - y1) * 0.45));
  return `M ${x1} ${y1} C ${x1} ${y1 + controlOffset}, ${x2} ${y2 - controlOffset}, ${x2} ${y2}`;
}
