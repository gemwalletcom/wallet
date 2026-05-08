package com.gemwallet.android.data.service.store.database

import androidx.sqlite.db.SimpleSQLiteQuery
import androidx.sqlite.db.SupportSQLiteQuery

data class SqlQuery(val sql: String, val args: List<Any>) {
    fun toSupportSQLiteQuery(): SupportSQLiteQuery = SimpleSQLiteQuery(sql, args.toTypedArray())
}

data class SqlClause(val sql: String, val args: List<Any> = emptyList()) {
    val isEmpty: Boolean get() = sql.isBlank()

    companion object {
        val EMPTY = SqlClause("")

        fun equalTo(column: String, value: Any): SqlClause =
            SqlClause("$column = ?", listOf(value))

        fun greaterThan(column: String, value: Any): SqlClause =
            SqlClause("$column > ?", listOf(value))

        fun inList(column: String, values: Collection<Any>): SqlClause {
            if (values.isEmpty()) return EMPTY
            val placeholders = values.joinToString(",") { "?" }
            return SqlClause("$column IN ($placeholders)", values.toList())
        }

        fun raw(sql: String, vararg args: Any): SqlClause =
            SqlClause(sql, args.toList())
    }
}

class SqlQueryBuilder(
    private val baseSql: String,
    private val baseArgs: List<Any> = emptyList(),
) {
    private val clauses = mutableListOf<SqlClause>()
    private var orderBy: String? = null
    private var limit: Int? = null

    fun where(clause: SqlClause) = apply {
        if (!clause.isEmpty) clauses.add(clause)
    }

    fun whereAll(clauses: Iterable<SqlClause>) = apply {
        clauses.forEach(::where)
    }

    fun orderBy(expr: String) = apply { orderBy = expr }

    fun limit(n: Int) = apply { limit = n }

    fun build(): SqlQuery {
        val sql = StringBuilder(baseSql)
        val args = mutableListOf<Any>().apply { addAll(baseArgs) }
        if (clauses.isNotEmpty()) {
            val baseHasWhere = baseSql.contains("WHERE", ignoreCase = true)
            sql.append(if (baseHasWhere) " AND " else " WHERE ")
            clauses.forEachIndexed { i, c ->
                if (i > 0) sql.append(" AND ")
                sql.append(c.sql)
                args.addAll(c.args)
            }
        }
        orderBy?.let { sql.append(" ORDER BY ").append(it) }
        limit?.let {
            sql.append(" LIMIT ?")
            args.add(it)
        }
        return SqlQuery(sql.toString(), args)
    }
}
