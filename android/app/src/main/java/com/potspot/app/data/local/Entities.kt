package com.potspot.app.data.local

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "strains")
data class StrainEntity(
    @PrimaryKey val id: String,
    val name: String,
    val strainType: String,
    val thcPercentage: Double?,
    val cbdPercentage: Double?,
    val description: String?,
    val color: String?,
    val smell: String?,
    val flavor: String?,
    val breeder: String?,
    val lineage: String?,
    val growingDifficulty: String?,
    val floweringTimeDays: Int?,
    val averageRating: Double?,
    val ratingCount: Int,
    val version: Int,
    val updatedAt: String,
    val isSaved: Boolean = false,    // True if user bookmarked this strain
)

@Entity(tableName = "terpenes")
data class TerpeneEntity(
    @PrimaryKey val id: Int,
    val name: String,
    val icon: String,
    val description: String?,
)

@Entity(tableName = "strain_terpenes")
data class StrainTerpeneCrossRef(
    @PrimaryKey(autoGenerate = true) val uid: Int = 0,
    val strainId: String,
    val terpeneId: Int,
)

@Entity(tableName = "effects")
data class EffectEntity(
    @PrimaryKey val id: Int,
    val name: String,
    val category: String,
)

@Entity(tableName = "strain_effects")
data class StrainEffectCrossRef(
    @PrimaryKey(autoGenerate = true) val uid: Int = 0,
    val strainId: String,
    val effectId: Int,
)
