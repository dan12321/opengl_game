#version 330 core

in vec2 TexCoord;
in float LightIntensity;
in float SpecularIntensity;
in vec3 FragPos;
in vec3 Normal;
uniform sampler2D texture0;
uniform vec3 lightColor;
uniform vec3 ambientColor;
uniform float specularStrength;
uniform float ambientColorIntensity;
uniform vec3 lightPosition;
uniform float lightStrength;
uniform vec3 cameraPosition;
uniform int shininess;

out vec4 FragColor;

void main()
{
    vec4 tex = texture2D(texture0, TexCoord);
    float dist = distance(FragPos, lightPosition);
    vec3 lightDir = normalize(lightPosition - FragPos);
    float diffuse = max(dot(Normal, lightDir), 0.0);
    vec3 cameraDir = normalize(cameraPosition - FragPos);
    vec3 reflectDir = reflect(-lightDir, Normal);
    float distanceFallOff = min(1.0, lightStrength / pow(dist, 2.0));
    float specularIntensity = distanceFallOff * pow(max(dot(cameraDir, reflectDir), 0.0), shininess);
    float lightIntensity = distanceFallOff * diffuse;
    vec4 ambient = vec4(ambientColorIntensity * ambientColor, 1.0);
    vec4 light = vec4(lightIntensity * lightColor, 1.0);
    vec4 specular = vec4(specularStrength * specularIntensity * lightColor, 1.0);
    FragColor = (ambient + light + specular) * tex;
}
