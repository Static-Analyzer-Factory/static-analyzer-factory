"""Neo4j graph exporter for SAF ProgramDatabase.

Exports PropertyGraph data from SAF into a Neo4j database for
Cypher-based graph exploration. Supports project-level scoping so
multiple programs can coexist in the same database.
"""

import json
from typing import Any

try:
    import neo4j as neo4j_driver

    HAS_NEO4J = True
except ImportError:
    HAS_NEO4J = False


class Neo4jExporter:
    """Export SAF analysis graphs to Neo4j.

    Requires the ``neo4j`` Python package::

        pip install neo4j

    Each export is scoped by a ``project`` name so multiple programs can
    coexist in the same Neo4j database. Use ``clear(project)`` to remove
    a single project or ``clear()`` to wipe everything.

    Args:
        uri: Neo4j Bolt URI (default: ``bolt://localhost:7687``).
    """

    def __init__(self, uri: str = "bolt://localhost:7687") -> None:
        if not HAS_NEO4J:
            raise ImportError(
                "neo4j driver not installed. Install with: pip install neo4j"
            )
        self.driver = neo4j_driver.GraphDatabase.driver(uri)

    def close(self) -> None:
        """Close the Neo4j driver connection."""
        self.driver.close()

    def clear(self, project: str | None = None) -> None:
        """Clear nodes and relationships from the database.

        Args:
            project: If given, only delete nodes belonging to this project.
                     If ``None``, delete everything.
        """
        with self.driver.session() as session:
            if project is None:
                session.run("MATCH (n) DETACH DELETE n")
            else:
                session.run(
                    "MATCH (n {project: $project}) DETACH DELETE n",
                    project=project,
                )

    def export_from_json(
        self, graphs_json: list[str], project: str = "default"
    ) -> dict[str, int]:
        """Export PropertyGraph JSON strings to Neo4j.

        Args:
            graphs_json: List of JSON strings, each a PropertyGraph.
            project: Project name to scope all nodes/edges under.

        Returns:
            dict with import statistics (graphs, nodes, edges).
        """
        stats: dict[str, int] = {"graphs": 0, "nodes": 0, "edges": 0}

        with self.driver.session() as session:
            self._create_indexes(session)

            for graph_str in graphs_json:
                graph = (
                    json.loads(graph_str)
                    if isinstance(graph_str, str)
                    else graph_str
                )
                self._import_graph(session, graph, project)
                stats["graphs"] += 1
                stats["nodes"] += len(graph.get("nodes", []))
                stats["edges"] += len(graph.get("edges", []))

        return stats

    def list_projects(self) -> list[str]:
        """Return all distinct project names in the database."""
        with self.driver.session() as session:
            result = session.run(
                "MATCH (n) WHERE n.project IS NOT NULL "
                "RETURN DISTINCT n.project AS project ORDER BY project"
            )
            return [record["project"] for record in result]

    def _create_indexes(self, session: Any) -> None:
        """Create Neo4j indexes for common query patterns."""
        indexes = [
            "CREATE INDEX IF NOT EXISTS FOR (n:Function) ON (n.name)",
            "CREATE INDEX IF NOT EXISTS FOR (n:Value) ON (n.id)",
            "CREATE INDEX IF NOT EXISTS FOR (n:Location) ON (n.id)",
            "CREATE INDEX IF NOT EXISTS FOR (n:Block) ON (n.id)",
            "CREATE INDEX IF NOT EXISTS FOR (n:Instruction) ON (n.id)",
        ]
        for idx in indexes:
            session.run(idx)

    def _import_graph(
        self, session: Any, graph: dict[str, Any], project: str
    ) -> None:
        """Import a single PropertyGraph into Neo4j."""
        graph_type = graph.get("graph_type", "unknown")
        nodes = graph.get("nodes", [])
        edges = graph.get("edges", [])

        if not nodes:
            return

        # Create nodes with project + graph_type scoping
        for node in nodes:
            labels = ":".join(node.get("labels", ["Node"]))
            props = dict(node.get("properties", {}))
            props["id"] = node["id"]
            props["project"] = project
            props["graph_type"] = graph_type
            session.run(
                f"CREATE (n:{labels}) SET n += $props",
                props=props,
            )

        # Create relationships scoped to same project
        for edge in edges:
            edge_type = edge.get("edge_type", "RELATED_TO")
            edge_props = dict(edge.get("properties", {}))
            edge_props["project"] = project
            edge_props["graph_type"] = graph_type
            session.run(
                f"MATCH (a {{id: $src, project: $project}}), "
                f"(b {{id: $dst, project: $project}}) "
                f"CREATE (a)-[r:{edge_type}]->(b) "
                f"SET r += $props",
                src=edge["src"],
                dst=edge["dst"],
                project=project,
                props=edge_props,
            )

    def cypher(self, query: str, **params: Any) -> list[dict[str, Any]]:
        """Execute a Cypher query and return results as list of dicts.

        Args:
            query: Cypher query string.
            **params: Query parameters.

        Returns:
            List of result records as dictionaries.
        """
        with self.driver.session() as session:
            result = session.run(query, **params)
            return [dict(record) for record in result]
