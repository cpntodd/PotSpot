package com.potspot.app.data.repository

import com.potspot.app.data.local.PotSpotDatabase
import com.potspot.app.data.local.StrainEntity
import com.potspot.app.data.remote.ApiClient
import kotlinx.coroutines.flow.Flow

class StrainRepository(private val db: PotSpotDatabase) {
    private val dao = db.strainDao()
    private val api = ApiClient.api

    // Local catalog (offline-first)
    fun getLocalStrains(): Flow<List<StrainEntity>> = dao.getAllStrains()

    fun searchLocalStrains(query: String, typeFilter: String?, sortBy: String): Flow<List<StrainEntity>> =
        dao.searchStrains(query, typeFilter, sortBy)

    fun getSavedStrains(): Flow<List<StrainEntity>> = dao.getSavedStrains()

    suspend fun getLocalStrainById(id: String): StrainEntity? = dao.getStrainById(id)

    // Sync: fetch full catalog from server and store locally
    suspend fun syncCatalog() {
        try {
            var page = 1L
            var hasMore = true
            val allStrains = mutableListOf<StrainEntity>()

            while (hasMore) {
                val response = api.getStrains(page = page, perPage = 50)
                allStrains.addAll(response.strains.map { it.toEntity() })
                hasMore = (page * 50) < response.total
                page++
            }

            // Replace unsaved strains with fresh data
            dao.deleteUnsavedStrains()
            dao.insertStrains(allStrains)
        } catch (e: Exception) {
            // Offline -- use cached data
        }
    }

    // Toggle save/unsave
    suspend fun toggleSaved(strainId: String, saved: Boolean) {
        dao.setSaved(strainId, saved)
        try {
            if (saved) api.saveStrain(strainId)
            else api.unsaveStrain(strainId)
        } catch (_: Exception) { }
    }

    // Map DTOs to entities
    private fun com.potspot.app.data.remote.StrainSummaryDto.toEntity() = StrainEntity(
        id = id,
        name = name,
        strainType = strainType,
        thcPercentage = thcPercentage,
        cbdPercentage = cbdPercentage,
        description = null,
        color = null,
        smell = null,
        flavor = null,
        breeder = null,
        lineage = null,
        growingDifficulty = null,
        floweringTimeDays = null,
        averageRating = averageRating,
        ratingCount = ratingCount,
        version = 0,
        updatedAt = "",
        isSaved = false,
    )
}
