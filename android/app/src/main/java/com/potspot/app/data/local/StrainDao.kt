package com.potspot.app.data.local

import androidx.room.*
import kotlinx.coroutines.flow.Flow

@Dao
interface StrainDao {
    // Catalog
    @Query("SELECT * FROM strains ORDER BY updatedAt DESC")
    fun getAllStrains(): Flow<List<StrainEntity>>

    @Query("SELECT * FROM strains WHERE id = :id")
    suspend fun getStrainById(id: String): StrainEntity?

    @Query("""
        SELECT * FROM strains WHERE
        name LIKE '%' || :query || '%'
        AND (:typeFilter IS NULL OR strainType = :typeFilter)
        ORDER BY
            CASE WHEN :sortBy = 'rating' THEN averageRating END DESC,
            CASE WHEN :sortBy = 'name' THEN name END ASC,
            CASE WHEN :sortBy = 'newest' THEN updatedAt END DESC
    """)
    fun searchStrains(
        query: String,
        typeFilter: String?,
        sortBy: String,
    ): Flow<List<StrainEntity>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertStrains(strains: List<StrainEntity>)

    @Query("UPDATE strains SET isSaved = :saved WHERE id = :strainId")
    suspend fun setSaved(strainId: String, saved: Boolean)

    @Query("SELECT * FROM strains WHERE isSaved = 1 ORDER BY updatedAt DESC")
    fun getSavedStrains(): Flow<List<StrainEntity>>

    @Query("DELETE FROM strains WHERE isSaved = 0")
    suspend fun deleteUnsavedStrains()

    // Terpenes
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertTerpenes(terpenes: List<TerpeneEntity>)

    @Query("SELECT * FROM terpenes ORDER BY name")
    fun getAllTerpenes(): Flow<List<TerpeneEntity>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertStrainTerpenes(crossRefs: List<StrainTerpeneCrossRef>)

    @Query("SELECT terpeneId FROM strain_terpenes WHERE strainId = :strainId")
    suspend fun getTerpeneIdsForStrain(strainId: String): List<Int>

    // Effects
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertEffects(effects: List<EffectEntity>)

    @Query("SELECT * FROM effects ORDER BY category, name")
    fun getAllEffects(): Flow<List<EffectEntity>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertStrainEffects(crossRefs: List<StrainEffectCrossRef>)

    @Query("SELECT effectId FROM strain_effects WHERE strainId = :strainId")
    suspend fun getEffectIdsForStrain(strainId: String): List<Int>
}
