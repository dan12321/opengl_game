#version 330 core
struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    int shininess;
};

in vec2 TexCoord;
in vec3 FragPos;
in vec3 Normal;

uniform sampler2D texture0;
uniform vec3 lightColor;
uniform vec3 lightPosition;
uniform float lightStrength;
uniform vec3 cameraPosition;
uniform Material material;

out vec4 FragColor;

void main()
{
    vec4 tex = texture2D(texture0, TexCoord);

    vec3 lightDir = normalize(lightPosition - FragPos);
    float lightDist = distance(FragPos, lightPosition);
    float distanceFallOff = min(1.0, lightStrength / pow(lightDist, 2.0));

    float diffuseIntensity = max(dot(Normal, lightDir), 0.0);
    vec3 diffuseColor = distanceFallOff * diffuseIntensity * material.diffuse;
    vec4 diffuse = vec4(diffuseColor * lightColor, 1.0);

    vec3 cameraDir = normalize(cameraPosition - FragPos);
    vec3 reflectDir = reflect(-lightDir, Normal);
    float specularIntensity = distanceFallOff * pow(max(dot(cameraDir, reflectDir), 0.0), material.shininess);
    vec4 specular = vec4(specularIntensity * lightColor * material.specular, 1.0);

    vec4 ambient = vec4(material.ambient, 1.0);

    FragColor = (ambient + diffuse + specular) * tex;
}
