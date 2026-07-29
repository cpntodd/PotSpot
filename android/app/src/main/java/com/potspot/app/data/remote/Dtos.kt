package com.potspot.app.data.remote

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class StrainListResponse(
    val strains: List<StrainSummaryDto>,
    val total: Long,
    val page: Long,
    @SerialName("per_page") val perPage: Long,
)

@Serializable
data class StrainSummaryDto(
    val id: String,
    val name: String,
    @SerialName("strain_type") val strainType: String,
    @SerialName("thc_percentage") val thcPercentage: Double? = null,
    @SerialName("cbd_percentage") val cbdPercentage: Double? = null,
    @SerialName("average_rating") val averageRating: Double? = null,
    @SerialName("rating_count") val ratingCount: Int = 0,
)

@Serializable
data class StrainDetailDto(
    val id: String,
    val name: String,
    val type: String,
    @SerialName("thc_percentage") val thcPercentage: Double? = null,
    @SerialName("cbd_percentage") val cbdPercentage: Double? = null,
    val description: String? = null,
    val color: String? = null,
    val smell: String? = null,
    val flavor: String? = null,
    val breeder: String? = null,
    val lineage: String? = null,
    @SerialName("growing_difficulty") val growingDifficulty: String? = null,
    @SerialName("flowering_time_days") val floweringTimeDays: Int? = null,
    @SerialName("average_rating") val averageRating: Double? = null,
    @SerialName("rating_count") val ratingCount: Int = 0,
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
    val version: Int = 1,
    val terpenes: List<TerpeneDto> = emptyList(),
    val effects: List<EffectDto> = emptyList(),
    @SerialName("primary_photo_url") val primaryPhotoUrl: String? = null,
)

@Serializable
data class TerpeneDto(
    val id: Int,
    val name: String,
    val icon: String,
    val description: String? = null,
)

@Serializable
data class EffectDto(
    val id: Int,
    val name: String,
    val category: String,
)

@Serializable
data class TokenResponse(
    @SerialName("access_token") val accessToken: String,
    @SerialName("refresh_token") val refreshToken: String,
    @SerialName("token_type") val tokenType: String,
    @SerialName("expires_in") val expiresIn: Int,
)

@Serializable
data class LoginRequest(
    val email: String,
    val password: String,
)

@Serializable
data class VaultResponse(
    @SerialName("private_strains") val privateStrains: List<StrainSummaryDto> = emptyList(),
    @SerialName("saved_strains") val savedStrains: List<StrainSummaryDto> = emptyList(),
)
