#!/bin/bash

set -euo pipefail

minecraft_map_renderer=~/.local/bin/minecraft-map-renderer
blockstate_registry=~/.local/share/minecraft-map-renderer/blockstate_registry.json

if [[ ! -f "$blockstate_registry" ]]; then
    echo "Blockstate registry not found at $blockstate_registry" >&2
    echo "Please edit this script to point to the correct blockstate registry file." >&2
    exit 1
fi

if [[ ! -f "$minecraft_map_renderer" ]]; then
    if which minecraft-map-renderer >/dev/null 2>&1; then
        minecraft_map_renderer=$(which minecraft-map-renderer)
    else
        echo "Minecraft map renderer not found at $minecraft_map_renderer" >&2
        echo "Please edit this script to point to the correct Minecraft map renderer binary." >&2
        exit 1
    fi
fi

to_bytes_le() {
    local num="$1"
    
    printf "\\$(printf '%03o' $((  num        & 0xFF )))"
    printf "\\$(printf '%03o' $(( (num >>  8) & 0xFF )))"
    printf "\\$(printf '%03o' $(( (num >> 16) & 0xFF )))"
    printf "\\$(printf '%03o' $(( (num >> 24) & 0xFF )))"
}

read_exact() {
    local length="$1"
    local timeout="${2:-30}"

    timeout "$timeout" dd bs=1 count="$length" 2>/dev/null

    if [[ $? -ne 0 ]]; then
        echo "Read timed out or failed" >&2
        return 1
    fi
}

from_bytes_le() {
    local n=0 b i=0
    for b in $(od -An -tu1 -v); do
        n=$(( (b << 8 * i) | n ))
        i=$((i + 1))
    done
    echo "$n"
}

send_rcon_packet() {
    local type="$1"
    local payload="$2"
    local serial="${3:-$RANDOM}"

    local length=$((4 + 4 + ${#payload} + 2))

    to_bytes_le "$length"
    to_bytes_le "$serial"
    to_bytes_le "$type"
    printf "%s" "$payload"
    printf "\0\0"
}

read_rcon_packet() {
    local expected_type="$1"
    local timeout="${3:-30}"

    local length=$(read_exact 4 "$timeout" | from_bytes_le)
    local serial=$(read_exact 4 "$timeout" | from_bytes_le)
    local type=$(read_exact 4 "$timeout" | from_bytes_le)
    local payload_length=$((length - 4 - 4 - 2))
    local payload=$(read_exact "$payload_length" "$timeout")
    read_exact 2 "$timeout"
    if [[ "$type" -ne "$expected_type" ]]; then
        return 1
    fi

    echo "$payload"
}

wait_until_response() {
    local expected="$1"
    local type="$2"
    local timeout="${3:-30}"

    if [[ -z "$expected" ]]; then
        read_rcon_packet "$type" "$timeout" <&3 >/dev/null
    else
        local payload=""
        while ! grep -q "$expected" <<< "$payload"; do
            payload=$(read_rcon_packet "$type" "$timeout" <&3 | tr -d '\000')
            if [[ $? -ne 0 ]]; then
                return 1
            fi
        done
    fi
}

# Run an RCON command on the server and optionally wait for a specific response
rcon_run() {
    local host="$1"
    local port="$2"
    local password="$3"
    local command="$4"
    local expected="$5"
    local timeout="${6:-30}"
    local tempfile=$(mktemp)

    exec 3<>/dev/tcp/"$host"/"$port"

    # Authenticate with the server using the RCON password
    send_rcon_packet 3 "$password" > "$tempfile"
    cat "$tempfile" >&3
    wait_until_response "" 2 "$timeout"
    
    # Send the command to the server
    send_rcon_packet 2 "$command" > "$tempfile"
    cat "$tempfile" >&3
    wait_until_response "$expected" 0 "$timeout"

    exec 3>&-
    rm -f "$tempfile"
}

# Render one world from the server
render_server_world() {
    local rcon_host="$1"
    local rcon_port="$2"
    local rcon_password="$3"
    local world_directory="$4"
    local output_directory="$5"
    local timeout="${6:-30}"

    echo "Preparing output directory..."
    mkdir -p "$output_directory"
    rm -rf "$output_directory"/*

    echo "Disabling automatic saving on the server..."
    rcon_run "$rcon_host" "$rcon_port" "$rcon_password" "save-off" "" "$timeout"
    rcon_run "$rcon_host" "$rcon_port" "$rcon_password" "save-all" "Saved the game" "$timeout"

    for file in "$world_directory"/dimensions/minecraft/*; do
        echo "Rendering dimension: $(basename "$file")"
        "$minecraft_map_renderer" --blockstate-registry "$blockstate_registry" \
            --input "$file/region" \
            --output "$output_directory/$(basename "$file")"
    done

    echo "Re-enabling automatic saving on the server..."
    rcon_run "$rcon_host" "$rcon_port" "$rcon_password" "save-on" "" "$timeout"
    
    echo "Finished rendering server!"
}

# Get property from the server with server.properties
get_server_property() {
    local property="$1"
    local server_directory="$2"

    if [[ -f "$server_directory/server.properties" ]]; then
        grep -E "^$property=" "$server_directory/server.properties" | cut -d'=' -f2-
    fi

    return 1
}

# Render a server, automatically get the password
render_server() {
    local server_directory="$1"
    local output_directory="$2"
    local timeout="${3:-30}"

    local rcon_host=$(get_server_property "server-ip" "$server_directory")
    local rcon_port=$(get_server_property "rcon.port" "$server_directory")
    local rcon_password=$(get_server_property "rcon.password" "$server_directory")

    if [[ -z "$rcon_host" ]]; then
        rcon_host="localhost"
    fi

    local world_directory="$server_directory/world"

    render_server_world "$rcon_host" "$rcon_port" "$rcon_password" "$world_directory" "$output_directory" "$timeout"
}

# Publish the rendered maps to git
publish_rendered_maps() {
    local output_directory="$1"
    local date=$(date +"%Y-%m-%d %H:%M:%S")
    local commit_message="${2:-"Update rendered maps ($date)"}"

    git -C "$output_directory" pull
    if [[ -n "$(git -C "$output_directory" status --porcelain)" ]]; then
        git -C "$output_directory" add .
        git -C "$output_directory" commit -m "$commit_message"
        git -C "$output_directory" push
        echo "Published rendered maps to git."
    else
        echo "No changes to commit."
    fi
}

# Please modify this script for your own server setup and preferences
echo "Please modify this script for your own server setup and preferences." >&2