package com.potspot.app.ui.screens

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.Person
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.potspot.app.data.local.StrainEntity
import com.potspot.app.data.repository.StrainRepository
import com.potspot.app.PotSpotApplication

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CatalogScreen(
    onStrainClick: (String) -> Unit,
    onVaultClick: () -> Unit,
    onProfileClick: () -> Unit,
) {
    val db = PotSpotApplication.instance.let { app ->
        com.potspot.app.data.local.PotSpotDatabase.getInstance(app)
    }
    val repository = remember { StrainRepository(db) }

    var searchQuery by remember { mutableStateOf("") }
    var filterType by remember { mutableStateOf<String?>(null) }
    val strains by repository.getLocalStrains().collectAsState(initial = emptyList())

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("PotSpot") },
                actions = {
                    IconButton(onClick = onVaultClick) {
                        Icon(Icons.Default.Favorite, contentDescription = "Vault")
                    }
                    IconButton(onClick = onProfileClick) {
                        Icon(Icons.Default.Person, contentDescription = "Profile")
                    }
                }
            )
        }
    ) { padding ->
        Column(modifier = Modifier.padding(padding)) {
            // Search bar
            OutlinedTextField(
                value = searchQuery,
                onValueChange = { searchQuery = it },
                label = { Text("Search strains...") },
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                singleLine = true,
            )
            // Type filter chips
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                listOf(null to "All", "sativa" to "Sativa", "indica" to "Indica", "hybrid" to "Hybrid").forEach { (type, label) ->
                    FilterChip(
                        selected = filterType == type,
                        onClick = { filterType = type },
                        label = { Text(label) },
                    )
                }
            }
            // Strain list
            val filtered = strains.filter {
                (searchQuery.isEmpty() || it.name.contains(searchQuery, ignoreCase = true)) &&
                (filterType == null || it.strainType == filterType)
            }
            LazyColumn(
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                items(filtered) { strain ->
                    StrainListItem(strain = strain, onClick = { onStrainClick(strain.id) })
                }
            }
        }
    }
}

@Composable
fun StrainListItem(strain: StrainEntity, onClick: () -> Unit) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(strain.name, style = MaterialTheme.typography.titleMedium)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(strain.strainType.replaceFirstChar { it.uppercase() })
                strain.thcPercentage?.let { Text("THC $it%") }
                strain.averageRating?.let {
                    Text("★ ${"%.1f".format(it)} (${strain.ratingCount})")
                }
            }
        }
    }
}
