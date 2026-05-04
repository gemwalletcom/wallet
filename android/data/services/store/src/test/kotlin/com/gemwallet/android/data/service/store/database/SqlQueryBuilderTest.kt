package com.gemwallet.android.data.service.store.database

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class SqlQueryBuilderTest {

    @Test
    fun emptyBuilder_returnsBaseSqlOnly() {
        val query = SqlQueryBuilder(baseSql = "SELECT * FROM posts").build()
        assertEquals("SELECT * FROM posts", query.sql)
        assertTrue(query.args.isEmpty())
    }

    @Test
    fun singleClause_addsWhenBaseHasNoWhere() {
        val query = SqlQueryBuilder(baseSql = "SELECT * FROM posts")
            .where(SqlClause.equalTo("title", "hello"))
            .build()
        assertEquals("SELECT * FROM posts WHERE title = ?", query.sql)
        assertEquals(listOf<Any>("hello"), query.args)
    }

    @Test
    fun singleClause_addsAndWhenBaseSqlAlreadyHasWhere() {
        val query = SqlQueryBuilder(baseSql = "SELECT * FROM posts WHERE published = 1")
            .where(SqlClause.equalTo("title", "hello"))
            .build()
        assertEquals("SELECT * FROM posts WHERE published = 1 AND title = ?", query.sql)
    }

    @Test
    fun multipleClauses_joinedWithAndPreservingOrder() {
        val query = SqlQueryBuilder(baseSql = "SELECT * FROM posts")
            .where(SqlClause.equalTo("authorId", 1))
            .where(SqlClause.greaterThan("views", 100))
            .where(SqlClause.inList("status", listOf("active", "pinned")))
            .build()
        assertEquals(
            "SELECT * FROM posts WHERE authorId = ? AND views > ? AND status IN (?,?)",
            query.sql,
        )
        assertEquals(listOf<Any>(1, 100, "active", "pinned"), query.args)
    }

    @Test
    fun emptyInList_isNoOp() {
        val query = SqlQueryBuilder(baseSql = "SELECT * FROM posts")
            .where(SqlClause.inList("tags", emptyList()))
            .build()
        assertEquals("SELECT * FROM posts", query.sql)
    }

    @Test
    fun orderByAndLimit_appendedAtEndWithLimitArgLast() {
        val query = SqlQueryBuilder(baseSql = "SELECT * FROM posts")
            .where(SqlClause.equalTo("authorId", 1))
            .orderBy("createdAt DESC")
            .limit(50)
            .build()
        assertEquals("SELECT * FROM posts WHERE authorId = ? ORDER BY createdAt DESC LIMIT ?", query.sql)
        assertEquals(listOf<Any>(1, 50), query.args)
    }

    @Test
    fun rawClause_passesArgsThrough() {
        val query = SqlQueryBuilder(baseSql = "SELECT * FROM posts")
            .where(SqlClause.raw("(title LIKE ? OR body LIKE ?)", "%hello%", "%world%"))
            .build()
        assertEquals("SELECT * FROM posts WHERE (title LIKE ? OR body LIKE ?)", query.sql)
        assertEquals(listOf<Any>("%hello%", "%world%"), query.args)
    }
}
