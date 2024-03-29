#! /bin/python3

# ----------------------------------------
# IMPORTS
# ----------------------------------------

import argparse
import json
import logging
import sqlite3

# ----------------------------------------
# GLOBALS
# ----------------------------------------

DEBUGGING = True

# ----------------------------------------
# FUNCTIONS
# ----------------------------------------

def configure ():
    parser = argparse.ArgumentParser (description = "Collect SQLite3 database schema information.")

    parser.add_argument ("--exclude",        help = "exclude tables by name", action = "append")
    parser.add_argument ("--include",        help = "include tables by name", action = "append")
    parser.add_argument ("--prefix-exclude", help = "exclude tables whose name starts with prefix", action = "append")
    parser.add_argument ("--prefix-include", help = "include tables whose name starts with prefix", action = "append")

    parser.add_argument ("--debug-dump-schema", help = "dump a JSON representation of the accumulated schema and exit",
                         action = "store_true")

    parser.add_argument ("database", help = "path to SQLite3 database")

    arguments = parser.parse_args ()

    return { "database":         arguments.database,
             "excludes":         arguments.exclude if arguments.exclude else [ ],
             "includes":         arguments.include if arguments.include else [ ],
             "exclude prefixes": arguments.prefix_exclude if arguments.prefix_exclude else [ ],
             "include prefixes": arguments.prefix_include if arguments.prefix_include else [ ],
             "debug dump schema": arguments.debug_dump_schema, }


def graph_of (schema):
    def graph_foreign_key (foreign_key):
        return (  foreign_key["from table name"] + ":" + foreign_key["from column name"]
                + " -> "
                + foreign_key["to table name"] + ":" + foreign_key["to column name"]
                + ";")


    def graph_table (table):
        def graph_table_name (table):
            return "<TR><TD BGCOLOR=\"gray\" COLSPAN=\"2\"><B>" + table["table name"] + "</B></TD></TR>"


        def graph_column (table_name, column):
            return (  "<TR>"
                    + "<TD PORT=\"" + column["column name"] + "\">"
                      + (("<I>" + column["column name"] + "</I>") if column["is pk"] else column["column name"])
                    + "</TD>"
                    + "<TD>" + column["type"] + "</TD>"
                    + "</TR>")


        return (  table["table name"] + " [label=<\n"
                + "    <TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\">\n"
                + "      " + graph_table_name (table) + "\n"
                + "\n".join ([ ("      " + graph_column (table["table name"], column)) for column in table["columns"] ]) + "\n"
                + "    </TABLE>>];")


    return (  "digraph schema {\n"
            + "  node[shape=none]\n\n"
            + "\n".join ([ ("  " + graph_table (table)) for table in schema["tables"] ]) + "\n\n"
            + "\n".join ([ ("  " + graph_foreign_key (foreign_key)) for foreign_key in schema["foreign keys"] ]) + "\n"
            + "}")


def schema_of (configuration):
    def columns (table, connection):
        def column (info, names):
            return { "column name": info[names["name"]],
                     "type":        info[names["type"]],
                     "nullable":    info[names["notnull"]] == 0,
                     "default":     None if info[names["dflt_value"]] == "" else info[names["dflt_value"]],
                     "is pk":       info[names["pk"]] == 1, }

        # Technically, there's a SQL injection attack lurking here: we should never use Python string operations to
        # composed SQL statements. However, the underlying DB-API library's parameter substitution mechanism cannot be
        # used to substitute in objects ... only values, and furthermore the table names were pulled from the database,
        # so the tables already exists. In other words: had there been a problem, it would've happened before this whole
        # program even started executing.
        #
        cursor = connection.execute ("PRAGMA table_info (\"" + table["table name"] + "\");")

        return [ column (info, names_of (cursor)) for info in cursor ]


    def foreign_keys (table, connection):
        def foreign_key (info, names):
            return { "table name":       info[names["table"]],
                     "from column name": info[names["from"]],
                     "to column name":   info[names["to"]], }

        # See the comment elsewhere about a potential SQL injection attack.
        #
        cursor = connection.execute ("PRAGMA foreign_key_list (\"" + table["table name"] + "\");")

        return [ foreign_key (info, names_of (cursor)) for info in cursor ]


    def hoist_foreign_keys (schema):
        def constructed (foreign_key, table_name):
            return { "from table name":  table_name,
                     "from column name": foreign_key["from column name"],
                     "to table name":    foreign_key["table name"],
                     "to column name":   foreign_key["to column name"], }

        return [ constructed (foreign_key, table["table name"])
                 for table in schema["tables"]
                 for foreign_key in table["foreign keys"] ]


    def names_of (cursor):
        return { description[0]: index for ( index, description ) in enumerate (cursor.description) }


    def tables (connection, configuration):
        cursor = connection.execute ("SELECT name FROM sqlite_master WHERE type=\"table\";")

        for result in cursor:
            table_name = result[names_of (cursor)["name"]]

            if configuration["includes"]:
                if table_name in configuration["includes"]:
                    yield { "table name": table_name, }

            else:
                if configuration["include prefixes"]:
                    if any ([ table_name.startswith (prefix) for prefix in configuration["include prefixes"] ]):
                        yield { "table name": table_name, }

                else:
                    if table_name in configuration["excludes"]:
                        pass

                    elif any ([ table_name.startswith (prefix) for prefix in configuration["exclude prefixes"] ]):
                        pass

                    else:
                        yield { "table name": table_name, }


    connection = sqlite3.connect (configuration["database"])
    schema     = { "tables": list (tables (connection, configuration)) }

    for table in schema["tables"]:
        table["columns"]      = columns (table, connection)
        table["foreign keys"] = foreign_keys (table, connection)

    schema["foreign keys"] = hoist_foreign_keys (schema)

    return schema

# ----------------------------------------
# MAIN PROCESSING
# ----------------------------------------

if __name__ == "__main__":
    # Configure the log system.
    #
    global LOGGER

    logging.basicConfig (format = ("------------------------------------------------------------------------\n"
                                   + "%(name)s:%(levelname)s:\n%(message)s\n"),
                         level = logging.DEBUG if DEBUGGING else logging.INFO)

    LOGGER = logging.getLogger (__name__)


if __name__ == "__main__":
    configuration = configure ()

    schema = schema_of (configuration)

    if configuration["debug dump schema"]:
        print (json.dumps (schema, indent = 4))

    else:
        print (graph_of (schema))

